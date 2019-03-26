use crate::configuration::Config;
use crate::errors::Error;
use crate::utils::read_file;
use crate::utils::write_to_fifo;
use std::fmt;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

pub struct Service {
    pub name: String,     // Name of the service
    pub srcpath: PathBuf, // Full path to a service physically
    pub dstpath: PathBuf, // Full path to a service symlink (can be empty if disabled)
    config: Config,       // Configuration that holds SvConfig which contains user configuration
}

pub struct Status {
    name: String,
    status: String,
    pid: u32,
    talive: u64,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            status: String::with_capacity(4),
            pid: 0,
            talive: 0,
        }
    }
}

// This implements displaying the status of the service
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.status == "down" {
            write!(
                f,
                "{}: {}: {}s, normally up",
                self.status, self.name, self.talive
            )
        } else {
            write!(
                f,
                "{}: {}: (pid {}) {}s",
                self.status, self.name, self.pid, self.talive
            )
        }
    }
}

impl Status {
    pub(crate) fn status(&mut self, s: &Service, l: bool) -> Result<&mut Self, Error> {
        let mut pidf: PathBuf = PathBuf::from(&s.dstpath);

        if !&pidf.exists() {
            return Err(Error::Disabled(s.name.clone()));
        }

        if l {
            pidf = Service::make_path(&s, "log/supervise/pid");
        } else {
            pidf = Service::make_path(&s, "supervise/pid");
        }

        // Get status of the service
        match read_file(&pidf) {
            Ok(b) => {
                if b.is_empty() {
                    self.status = "down".to_string();
                    self.pid = 0
                } else {
                    self.pid = match b.trim().parse::<u32>() {
                        Ok(b) => b,
                        Err(e) => return Err(Error::ParseInt(e)),
                    };
                    self.status = "run".to_string()
                }
            }
            Err(e) => return Err(e),
        };

        // Get the modification time of the pid file which can be used
        // to know the time the process was alive
        let time = match std::fs::metadata(&pidf) {
            Ok(t) => t,
            Err(e) => return Err(Error::Modified(pidf, e)),
        };

        self.talive = match time.modified().unwrap().elapsed() {
            Ok(t) => t.as_secs(),
            Err(e) => return Err(Error::SystemTime(e)),
        };

        // Get our name from the name of the Service given to us
        self.name = s.name.clone();

        Ok(self)
    }
}

impl Service {
    pub(crate) fn new(c: Config) -> Service {
        Service {
            name: "".to_string(),
            srcpath: PathBuf::new(),
            dstpath: PathBuf::new(),
            config: c,
        }
    }

    /*
     * Use the name of the type and get srcpath and dstpath
     * Call them after running Service::New to get the paths from
     * the configuration file.
     */
    pub(crate) fn get_paths(&mut self) -> Result<&mut Self, Error> {
        let mut srcpath: PathBuf = PathBuf::from(&self.config.config.svdir);
        let mut dstpath: PathBuf = PathBuf::from(&self.config.config.lndir);

        if !srcpath.is_dir() {
            return Err(Error::NeedsDir(srcpath, self.name.clone()));
        }
        srcpath.push(&self.name);

        if !dstpath.is_dir() {
            return Err(Error::NeedsDir(dstpath, self.name.clone()));
        }
        dstpath.push(&self.name);

        self.srcpath = srcpath;
        self.dstpath = dstpath;

        Ok(self)
    }

    pub(crate) fn rename(&mut self, n: String) -> Result<&mut Self, Error> {
        self.name = n.clone();

        // Check if our srcpath is the same as in the config
        if self.srcpath.parent().unwrap() == self.config.config.svdir {
            self.srcpath.pop();
        }

        if self.dstpath.parent().unwrap() == self.config.config.lndir {
            self.dstpath.pop();
        }

        self.srcpath.push(n.clone());
        self.dstpath.push(n);

        Ok(self)
    }

    // Function to make a path to Fifo
    fn make_path(&self, s: &str) -> PathBuf {
        let mut p = PathBuf::from(&self.dstpath);
        p.push(s);
        return p;
    }

    pub(crate) fn stop(&self) -> Result<(), Error> {
        let target: PathBuf = PathBuf::from(&self.dstpath);

        if !target.exists() {
            return Err(Error::NotEnabled(self.name.clone()));
        }

        match self.signal("d") {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        let buffer = match read_file(&Self::make_path(&self, "supervise/stat")) {
            Ok(b) => b,
            Err(e) => return Err(e),
        };

        if buffer != "down\n" {
            return Err(Error::CouldNotDisable(self.name.clone()));
        }

        // If we reached here we
        // 1. Wrote to the fifo
        // 2. Read the stat file
        // 3. Confirmed the service is down
        Ok(())
    }

    pub(crate) fn disable(&self) -> Result<(), Error> {
        let target: PathBuf = PathBuf::from(&self.dstpath);

        if !target.exists() {
            return Err(Error::Disabled(self.name.clone()));
        }

        match Self::stop(&self) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        match std::fs::remove_file(&target) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Remove(target, e)),
        }
    }

    pub(crate) fn signal(&self, s: &str) -> Result<(), Error> {
        let target: PathBuf = PathBuf::from(&self.dstpath);

        if !target.exists() {
            return Err(Error::NotEnabled(self.name.clone()));
        }

        match write_to_fifo(Self::make_path(&self, "supervise/control"), s) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // Create a symlink from the srcpath to the dstpath
    pub(crate) fn enable(&self) -> Result<(), Error> {
        let source: PathBuf = PathBuf::from(&self.srcpath);
        let target: PathBuf = PathBuf::from(&self.dstpath);

        if !&source.exists() {
            return Err(Error::NotExist(self.name.clone(), source));
        }

        // Check if service is already enabled (is a symlink)
        match std::fs::symlink_metadata(&target) {
            Ok(v) => {
                // Our target can't exist as a directory
                if v.is_dir() {
                    return Err(Error::IsDir(target));
                }

                // Our target can't exist as a file
                if v.is_file() {
                    return Err(Error::IsFile(target));
                }

                // Our target can be a symlink
                if v.file_type().is_symlink() {
                    let r: PathBuf = std::fs::read_link(&target).unwrap();

                    // But it must be a symlink point to our srcpath
                    if r == source {
                        return Err(Error::Enabled(self.name.to_string()));
                    }

                    // Otherwise it is a symlink to somewhere else we don't control
                    return Err(Error::Mismatch(target, self.name.clone()));
                }
            }
            Err(_) => (),
        }

        // Try to symlink, the most common error is lack of permissions
        match symlink(&source, &target) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Link(source, target, e)),
        }
    }
}
