extern crate toml;
use std::path::Path;
use serde::Deserialize;
use std::io::Read;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(String),
    DeToml(String),
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

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "path: {}", self.path);
        write!(f, "{}", self.config)
    }
}

impl fmt::Display for SvConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "servicedir path: {}", self.service_dir_path)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub path: String,
    pub config: SvConfig,
}

#[derive(Debug, Deserialize)]
pub struct SvConfig {
    pub service_dir_path: String,
}

/*
 * Searches in system paths for a config file that is readable
 */
pub fn find() -> Option<String> {
    let paths = vec![
        Path::new("/run/svctrl/config.toml"),
        Path::new("/etc/svctrl/config.toml"),
        Path::new("/usr/share/svctrl/config.toml"),
    ];

    for path in paths.iter() {
        if path.is_file() {
            return Some(path.to_str().unwrap().to_string());
        }
    }
    None
}

impl Default for SvConfig {
    fn default() -> Self {
        Self {
            service_dir_path: "".to_string()
        }
    }
}

impl Config {

    /*
     * Returns a given config
     */
    pub(crate) fn open(&mut self) -> Result<&mut Self, Error> {
        let mut config_file = std::fs::OpenOptions::new().read(true).open(&self.path)?;
        let mut config_string = String::new();

        config_file.read_to_string(&mut config_string)?;

        let config_toml: SvConfig = toml::from_str(&config_string)?;

        self.config = config_toml;

        Ok(self)
    }

}
