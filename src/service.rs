use crate::configuration::Config;
use crate::errors::Error;
use crate::utils::read_file;
use crate::utils::write_to_fifo;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

pub struct Service {
    pub name: String,     // Name of the service
    pub srcpath: PathBuf, // Full path to a service physically
    pub dstpath: PathBuf, // Full path to a service symlink (can be empty if disabled)
    config: Config,       // Configuration that holds SvConfig which contains user configuration
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

    //    pub(crate) fn status(self) -> Result<String, Error> {
    //        let target: PathBuf = self.dstpath;
    //
    //        if !&source.exists() {
    //            return Err(Error::Disabled(self.name));
    //        }
    //    }

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
