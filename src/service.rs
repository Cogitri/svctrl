use crate::configuration::Config;
use crate::errors::Error;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

pub struct Service {
    name: String,     // Name of the service
    srcpath: PathBuf, // Full path to a service physically
    dstpath: PathBuf, // Full path to a service symlink (can be empty if disabled)
    config: Config,   // Configuration that holds SvConfig which contains user configuration
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

    // Create a symlink from the srcpath to the dstpath
    pub(crate) fn enable(self) -> Result<(), Error> {
        let source: PathBuf = self.srcpath;
        let target: PathBuf = self.dstpath;

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
        match symlink(source, target) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
