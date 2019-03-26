extern crate toml;
use crate::errors::Error;
use serde::Deserialize;
use std::fmt;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "path = '{}'", self.path.to_str().unwrap());
        write!(f, "{}", self.config)
    }
}

impl fmt::Display for SvConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "svdir = '{}'", self.svdir.to_str().unwrap());
        write!(f, "lndir = '{}'", self.lndir.to_str().unwrap())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub path: PathBuf,
    pub config: SvConfig,
}

/*
 * These are the configuration options that are available to
 * be placed in the config.toml file
 */
#[derive(Debug, Deserialize, Clone)]
pub struct SvConfig {
    pub svdir: PathBuf, // Path to where directories live
    pub lndir: PathBuf, // Path to where directories are linked to
}

/*
 * Searches in system paths for a config file that is readable
 */
pub fn find() -> Option<PathBuf> {
    let paths = vec![
        Path::new("/run/svctrl/config.toml"),
        Path::new("/etc/svctrl/config.toml"),
        Path::new("/usr/share/svctrl/config.toml"),
    ];

    for path in paths.iter() {
        if path.is_file() {
            return Some(path.to_path_buf());
        }
    }
    None
}

impl Default for SvConfig {
    fn default() -> Self {
        Self {
            svdir: PathBuf::new(),
            lndir: PathBuf::new(),
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
