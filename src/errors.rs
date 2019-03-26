use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Path ({}) of service ({}) needs to be a directory!", _0, _1)]
    NeedsDir(String, String), // The Path given is invalid
    #[fail(display = "Service ({}) is already enabled", _0)]
    Enabled(String), // The Service is already enabled
    #[fail(display = "Service ({}) is already disabled", _0)]
    Disabled(String), // The Service is already disabled
    #[fail(display = "Service ({}) is not enabled", _0)]
    NotEnabled(String),
    #[fail(
        display = "Path ({}) of service ({}) is claimed by another service",
        _0, _1
    )]
    Mismatch(String, String), // The dstpath is claimed by another service
    #[fail(display = "Path ({}) is a directory", _0)]
    IsDir(String), // The dstpath is a directory
    #[fail(display = "Path ({}) is a file", _0)]
    IsFile(String), // The dstpath is a file
    #[fail(display = "{}", _0)]
    Io(String),
    #[fail(display = "Failed to deserialize config TOML! Error: {}", _0)]
    DeToml(String),
    #[fail(display = "Could not disable service ({}) by writing to fifo!", _0)]
    CouldNotDisable(String),

    // When the service is not in the srcpath
    #[fail(display = "Service ({}) not available on '{}'!", _0, _1)]
    NotExist(String, String),

    #[fail(display = "Could not open '{}'! Error: {}", _0, _1)]
    Open(String, std::io::Error),
    #[fail(display = "Could not write to '{}'! Error: {}", _0, _1)]
    Write(String, std::io::Error),
    #[fail(display = "Could not read '{}'! Error: {}", _0, _1)]
    Read(String, std::io::Error),

    // Used by enable
    #[fail(display = "Could not link '{}' to '{}'! Error: {}", _0, _1, _2)]
    Link(String, String, std::io::Error),

    // Used by disable
    #[fail(display = "Could not remove file on '{}'! Error: {}", _0, _1)]
    Remove(String, std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::DeToml(e.to_string())
    }
}
