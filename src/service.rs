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
    pub(crate) fn new(a: String, b: Config) -> Service {
        Service {
            name: a,
            srcpath: PathBuf::new(),
            dstpath: PathBuf::new(),
            config: b,
        }
    }

    /*
     * Use the name of the type and get srcpath and dstpath
     * Call them after running Service::New to get the paths from
     * the configuration file.
     */
    pub(crate) fn get_paths(&mut self) -> Result<&mut Self, Error> {
        let mut srcpath: PathBuf = PathBuf::new();
        let mut dstpath: PathBuf = PathBuf::new();

        srcpath.push(&self.config.config.svdir);
        if !srcpath.is_dir() {
            return Err(Error::NeedsDir(
                self.name.clone(),
                srcpath.into_os_string().into_string().unwrap(),
            ));
        }
        srcpath.push(&self.name);

        dstpath.push(&self.config.config.lndir);
        if !dstpath.is_dir() {
            return Err(Error::NeedsDir(
                self.name.clone(),
                dstpath.into_os_string().into_string().unwrap(),
            ));
        }
        dstpath.push(&self.name);

        self.srcpath = srcpath;
        self.dstpath = dstpath;

        Ok(self)
    }

    // Function to make a path to Fifo
    fn make_control_path(&self) -> PathBuf {
        let mut p = self.dstpath.clone();

        // Build path to the fifo
        p.push("supervise/control");

        return p;
    }

    // Function to make path to stat file
    fn make_stat_path(&self) -> PathBuf {
        let mut p = self.dstpath.clone();

        p.push("supervise/stat");

        return p;
    }

    pub(crate) fn stop(self) -> Result<(), Error> {
        let target: PathBuf = self.dstpath.clone();

        if !target.exists() {
            return Err(Error::NotEnabled(self.name));
        }

        match write_to_fifo(Self::make_control_path(&self), String::from("s")) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        let buffer = match read_file(Self::make_stat_path(&self)) {
            Ok(b) => b,
            Err(e) => return Err(e),
        };

        if buffer != "down" {
            return Err(Error::CouldNotDisable(self.name));
        }

        // If we reached here we
        // 1. Wrote to the fifo
        // 2. Read the stat file
        // 3. Confirmed the service is down
        Ok(())
    }

    pub(crate) fn disable(self) -> Result<(), Error> {
        let target: PathBuf = self.dstpath.clone();

        if !target.exists() {
            return Err(Error::Disabled(self.name));
        }

        match Self::stop(self) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        match std::fs::remove_file(&target) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Remove(
                target.into_os_string().into_string().unwrap(),
                e,
            )),
        }
    }

    // Create a symlink from the srcpath to the dstpath
    pub(crate) fn enable(self) -> Result<(), Error> {
        let source: PathBuf = self.srcpath;
        let target: PathBuf = self.dstpath;

        if !&source.exists() {
            return Err(Error::NotExist(
                self.name,
                source.into_os_string().into_string().unwrap(),
            ));
        }

        // Check if service is already enabled (is a symlink)
        match std::fs::symlink_metadata(&target) {
            Ok(v) => {
                // Our target can't exist as a directory
                if v.is_dir() {
                    return Err(Error::IsDir(target.into_os_string().into_string().unwrap()));
                }

                // Our target can't exist as a file
                if v.is_file() {
                    return Err(Error::IsFile(
                        target.into_os_string().into_string().unwrap(),
                    ));
                }

                // Our target can be a symlink
                if v.file_type().is_symlink() {
                    let r: PathBuf = std::fs::read_link(&target).unwrap();

                    // But it must be a symlink point to our srcpath
                    if r == source {
                        return Err(Error::Enabled(self.name.to_string()));
                    }

                    // Otherwise it is a symlink to somewhere else we don't control
                    return Err(Error::Mismatch(
                        self.name.to_string(),
                        target.into_os_string().into_string().unwrap(),
                    ));
                }
            }
            Err(_) => (),
        }

        // Try to symlink, the most common error is lack of permissions
        match symlink(&source, &target) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Link(
                source.into_os_string().into_string().unwrap(),
                target.into_os_string().into_string().unwrap(),
                e,
            )),
        }
    }
}
