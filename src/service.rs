use crate::configuration::Config;
use crate::errors::Error;
use crate::utils::read_file;
use crate::utils::write_to_fifo;
use std::fmt::Result as fmtResult;
use std::fmt::{Display, Formatter};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

/// Represents a service directory by runit
pub struct Service {
    /// Name of the directory where the service directory is
    pub name: String,
    /// Path where the service directory is located physically.
    pub srcpath: PathBuf,
    /// Path where the service directory is symlinked too but can be the same as srcpath
    pub dstpath: PathBuf,
    /// Struct that holds the configuration that is set on the Config.toml and where Config.toml
    /// is
    config: Config,
}

/// Represents the status of a service, used by status
pub struct Status {
    /// Name of the service, which is the directory
    name: String,
    /// Status of the service either down or run available under supervise/stat
    /// but supervise/pid is used instead
    status: String,
    /// Pid of the main process of the service available under supervise/pid
    pid: u32,
    /// Time in seconds since the service is either alive or dead, more specifically
    /// it is the time until the last change to supervise/pid since it changes whenever
    /// the state of the main service process in the system changes.
    talive: u64,
}

/// Default implementation of status, it is made manually instead
/// of using #[derive(Default)] to create a status String with a
/// sane capacity
impl Default for Status {
    fn default() -> Self {
        Self {
            name: String::new(),
            status: String::with_capacity(4),
            pid: 0,
            talive: 0,
        }
    }
}

/// `fmt::Display` for Status, formatted according to the output of runit's sv status
///
/// # Remarks
///
/// This is meant to completely match the output of 'sv status' that is present on
/// Void Linux.
impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
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
    /// Stores the values of a service into a Status struct
    ///
    /// # Arguments
    ///
    /// * `s` - A Service type that holds path where the service is linked and its name
    /// * `l` - Boolean indicating whether we are looking at a normal service or a logging
    /// subservice, if true then the paths where information is looked for is prefixed with log/
    ///
    /// # Remarks
    ///
    /// This function reads supervised/pid instead of supervise/stat to see if the service
    /// is active or not. If the supervise/pid is empty then the service is considered to
    /// be down.
    ///
    /// .elapsed is used for checking against SystemTime not a monotonic function which
    /// can yield inconsistent results in the system.
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
                    self.pid = b.trim().parse::<u32>()?;
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

    pub(crate) fn pretty_print(&self, log: bool) {
        if !log {
            println!("Name: {}", self.name);
            println!("  -> status: {}", self.status);
            if self.status == "up" {
                println!("  -> pid: {}", self.pid);
            }
            println!("  -> time {}: {}s", self.status, self.talive);
        } else {
            println!("  -> log:");
            println!("    -> status: {}", self.status);
            if self.status == "up" {
                println!("    -> pid: {}", self.pid);
            }
            println!("    -> time {}: {}s", self.status, self.talive);
        }
    }
}

impl Service {
    /// Implementation of new for Service takes a Config struct
    /// and returns all other values as empty String and `PathBuf`
    ///
    /// # Arguments
    ///
    /// * `c` - Config struct to be put into the struct on the config key
    ///
    /// # Example
    ///
    /// ```
    /// let conf = configuration::Config {
    ///     path: PathBuf::new("/etc/svctrl"),
    ///     config: Default::default(),
    /// };
    ///
    /// let sv: service::Service = service::Service::new(conf));
    /// ```
    pub(crate) fn new(c: Config) -> Self {
        Self {
            name: String::new(),
            srcpath: PathBuf::new(),
            dstpath: PathBuf::new(),
            config: c,
        }
    }

    /// Returns the struct given with srcpath and dstpath filled in
    ///
    /// # Remarks
    ///
    /// This function requires that the svdir and lndir values be set
    /// But it doesn't check for them.
    pub(crate) fn get_paths(&mut self) -> Result<&mut Self, Error> {
        self.srcpath = PathBuf::from(&self.config.svdir);
        self.dstpath = PathBuf::from(&self.config.lndir);

        self.srcpath.push(&self.name);
        self.dstpath.push(&self.name);

        Ok(self)
    }

    /// Returns the Struct itself with name changed and srcpath and dstpath adapted
    ///
    /// # Arguments
    ///
    /// * `n` - String representing the name that the service should have
    ///
    /// # Remarks
    ///
    /// This function re-uses the values in the config.svdir and config.lndir without
    /// reloading or performing any checks.
    ///
    /// # Example
    ///
    /// ```
    /// let conf = configuration::Config {
    ///     path: PathBuf::new("/etc/svctrl"),
    ///     config: Default::default(),
    /// };
    ///
    /// let sv: service::Service = service::Service::new(conf));
    ///
    /// for name in ["a", "b", "c"].iter() {
    ///     sv.rename(name)?;
    ///     println!("{:?}", sv);
    /// };
    /// ```
    pub(crate) fn rename(&mut self, n: String) -> Result<&mut Self, Error> {
        self.name = n.clone();
        self.srcpath = self.config.svdir.join(n.clone());
        self.dstpath = self.config.lndir.join(n);

        Ok(self)
    }

    /// Returns a `PathBuf` representing the path given prefixed with the dstpath
    /// of the service.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that holds the path to suffix to dstpath
    ///
    /// # Example
    /// ```
    /// let conf = configuration::Config {
    ///     path: PathBuf::new("/etc/svctrl"),
    ///     config: Default::default(),
    /// };
    ///
    /// let sv = Service {
    ///     name: "example".to_string(),
    ///     srcpath: PathBuf::new("/etc/sv"),
    ///     dstpath: PathBuf::new("/var/service"),
    ///     config: conf,
    /// };
    ///
    /// let path = Service::make_path(&s, "supervise/pid");
    /// assert_eq!(path, sv.config.lndir.join("supervise/pid")
    /// ```
    fn make_path(&self, s: &str) -> PathBuf {
        let mut p = PathBuf::from(&self.dstpath);
        p.push(s);
        p
    }

    pub(crate) fn stop(&self) -> Result<(), Error> {
        let target: PathBuf = PathBuf::from(&self.dstpath);

        if !target.exists() {
            return Err(Error::NotEnabled(self.name.clone()));
        }

        self.signal("d")?;

        let mut enabled: bool = true;

        // Try 5 times in a loop to read the supervise/stat file
        for _ in 1..5 {
            let buffer = read_file(&Self::make_path(&self, "supervise/stat"))?;

            if buffer == "down\n" {
                enabled = false;
                break;
            }

            // Wait for 0.5 second
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        if enabled {
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
        };

        Self::stop(&self)?;

        match std::fs::remove_file(&target) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Remove(target, e)),
        }
    }

    /// Writes a string to a fifo of a service
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice representing the text to be written
    ///
    /// # Remarks
    ///
    /// It does not check if the signal was consumed successfully by runsv only if the
    /// write was successfull
    ///
    /// # Example
    /// ```
    /// let conf = configuration::Config {
    ///     path: PathBuf::new("/etc/svctrl"),
    ///     config: Default::default(),
    /// };
    ///
    /// let sv = Service {
    ///     name: "example".to_string(),
    ///     srcpath: PathBuf::new("/etc/sv"),
    ///     dstpath: PathBuf::new("/var/service"),
    ///     config: conf,
    /// };
    ///
    /// // Send the service a down signal
    /// match sv.signal("d") {
    ///     Ok(_) => (),
    ///     Err(e) => Err(e),
    /// };
    /// ```
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
        if let Ok(v) = std::fs::symlink_metadata(&target) {
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

        // Try to symlink, the most common error is lack of permissions
        match symlink(&source, &target) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Link(source, target, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;

    use super::*;
    use crate::configuration::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup() -> Service {
        let tmpdir = tempfile::tempdir().unwrap();

        fs::create_dir_all(&tmpdir.path()).unwrap();
        fs::create_dir_all(&tmpdir.path().join("src")).unwrap();
        fs::create_dir_all(&tmpdir.path().join("dst")).unwrap();

        let test_conf = Config {
            path: None,
            svdir: tmpdir.path().join("src"),
            lndir: tmpdir.path().join("dst"),
        };

        let test_service = Service {
            name: "test".to_string(),
            srcpath: PathBuf::from(tmpdir.path().join("src")),
            dstpath: PathBuf::from(tmpdir.path().join("dst")),
            config: test_conf,
        };

        return test_service;
    }

    #[test]
    fn test_setup() {
        setup();
    }

    #[test]
    fn test_make_path() {
        let t = setup();

        assert_eq!(
            &Service::make_path(&t, "a/dir"),
            &t.config.lndir.join("a/dir")
        );

        assert_eq!(&Service::make_path(&t, "a"), &t.config.lndir.join("a"));

        assert_eq!(&Service::make_path(&t, "/t"), &t.config.lndir.join("/t"));
    }

    #[test]
    fn test_rename() {
        let mut t = setup();

        for x in ["test", "test1", "test2"].iter() {
            let n = x.to_string();

            t.rename(n.clone()).unwrap();
            assert_eq!(&t.srcpath, &t.config.svdir.join(&n));
            assert_eq!(&t.dstpath, &t.config.lndir.join(&n));
        }
    }
}
