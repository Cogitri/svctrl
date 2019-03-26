use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Path ({}) of service ({}) needs to be a directory!", _0, _1)]
    NeedsDir(String, String), // The Path given is invalid
    #[fail(display = "Service ({}) is already enabled", _0)]
    Enabled(String), // The Service is already enabled
    #[fail(
        display = "Path ({}) of service ({}) is claimed by another service",
        _0, _1
    )]
    Mismatch(String, String), // The dstpath is claimed by another service
    #[fail(display = "Path ({}) is a directory", _0)]
    IsDir(String), // The dstpath is a directory
    #[fail(display = "Path ({}) is a file", _0)]
    IsFile(String), // The dstpath is a file
    #[fail(display = "Failed to make symlink! Error: {}", _0)]
    Link(String),
    #[fail(display = "Failed to deserialize config TOML! Error: {}", _0)]
    DeToml(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Link(e.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::DeToml(e.to_string())
    }
}
