use crate::configuration::Config;
use std::fmt;
use std::os::unix::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    NotValid(String, String), // The Path given is invalid
    Enabled(String),          // The Service is already enabled
    Mismatch(String, String), // The dstpath is claimed by another service
    IsDir(String),            // The dstpath is a directory
    IsFile(String),           // The dstpath is a file
    Io(String),
    NoPerm(String), // We don't have permissions to create a symbolic link on dstpath
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Error::NotValid(e, s) => write!(f, "Service ({}) location ({}) is not valid", e, s),
            Error::Enabled(e) => write!(f, "Service '{}' already enabled", e),
            Error::IsDir(e) => write!(f, "Service location ({}) is a directory", e),
            Error::IsFile(e) => write!(f, "Service location ({}) is a file", e),
            Error::Mismatch(e, s) => write!(
                f,
                "Path of service '{}' points to mistmatched path ({})",
                e, s
            ),
            Error::Io(e) => f.write_str(e),
            Error::NoPerm(e) => write!(f, "We don't have permissions on {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NotValid(_, _) => "Service location is not valid",
            Error::Enabled(_) => "Service is already enabled",
            Error::Mismatch(_, _) => "The dstpath of the service is claimed by another service",
            Error::IsFile(_) => "Service location is a directory",
            Error::IsDir(_) => "Service location is a file",
            Error::Io(_) => "Io problems",
            Error::NoPerm(_) => "No permission to make symbolic link",
        }
    }
}

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
            return Err(Error::NotValid(
                self.name.clone(),
                srcpath.into_os_string().into_string().unwrap(),
            ));
        }
        srcpath.push(&self.name);

        dstpath.push(&self.config.config.lndir);
        if !dstpath.is_dir() {
            return Err(Error::NotValid(
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

        // Check if we have permission to write on the parent directory
        if let Some(perms) = target.parent() {
            match std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(perms)
            {
                Ok(_) => (),
                Err(_) => {
                    return Err(Error::NoPerm(
                        perms.to_owned().into_os_string().into_string().unwrap(),
                    ));
                }
            }
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
        match fs::symlink(source, target) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e.into()),
        }
    }
}
