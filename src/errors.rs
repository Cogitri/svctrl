use failure::Fail;
use std::path::PathBuf;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Service ({}) is already enabled", _0)]
    Enabled(String), // The Service is already enabled
    #[fail(display = "Service ({}) is disabled", _0)]
    Disabled(String), // The Service is already disabled
    #[fail(display = "Service ({}) is not enabled", _0)]
    NotEnabled(String),
    #[fail(
        display = "Path {:#?} of service '{}' is claimed by another service",
        _0, _1
    )]
    Mismatch(PathBuf, String), // The dstpath is claimed by another service
    #[fail(display = "Path {:#?} is a directory", _0)]
    IsDir(PathBuf), // The dstpath is a directory
    #[fail(display = "Path {:#?} is a file", _0)]
    IsFile(PathBuf), // The dstpath is a file
    #[fail(display = "{}", _0)]
    Io(String),
    #[fail(display = "{}", _0)]
    ParseInt(std::num::ParseIntError),
    #[fail(display = "{}", _0)]
    SystemTime(std::time::SystemTimeError),
    #[fail(display = "Failed to deserialize config TOML! Error: {}", _0)]
    DeToml(String),
    #[fail(display = "Could not disable service ({}) by writing to fifo!", _0)]
    CouldNotDisable(String),

    // When the service is not in the srcpath
    #[fail(display = "Service ({}) not available on {:#?}'!", _0, _1)]
    NotExist(String, PathBuf),

    // When open() is called but the path is None
    #[fail(display = "open() was called on a configuration but Path is None")]
    ConfNone,

    #[fail(display = "Could not open {:#?}! Error: {}", _0, _1)]
    Open(PathBuf, std::io::Error),
    #[fail(display = "Could not write to {:#?}! Error: {}", _0, _1)]
    Write(PathBuf, std::io::Error),
    #[fail(display = "Could not read {:#?}! Error: {}", _0, _1)]
    Read(PathBuf, std::io::Error),

    // Used by enable
    #[fail(display = "Could not link {:#?} to {:#?}! Error: {}", _0, _1, _2)]
    Link(PathBuf, PathBuf, std::io::Error),

    // Used by disable
    #[fail(display = "Could not remove file on {:#?}! Error: {}", _0, _1)]
    Remove(PathBuf, std::io::Error),

    // Used by status
    #[fail(display = "Could not read mtime of {:#?}! Error: {}", _0, _1)]
    Modified(PathBuf, std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Self {
        Error::SystemTime(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::DeToml(e.to_string())
    }
}
