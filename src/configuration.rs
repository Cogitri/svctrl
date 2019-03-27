extern crate toml;
use crate::errors::Error;
use serde::Deserialize;
use std::fmt;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

/// fmt::Display for Config, showing in the TOML format the configuration is written in
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.path.is_some() {
            writeln!(
                f,
                "path = '{}'",
                self.path.as_ref().unwrap().to_str().unwrap()
            );
        }
        write!(f, "{}", self.config)
    }
}

/// fmt::Display implementation for SvConfig, showing in the TOML format the configuration is
/// written in
impl fmt::Display for SvConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "svdir = '{}'", self.svdir.to_str().unwrap());
        write!(f, "lndir = '{}'", self.lndir.to_str().unwrap())
    }
}

/// Holds the location of a config and a struct that represents
/// the fields that can be set in the config itself
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Path to where the config is located for opening, reading and writing
    /// it is an option because it can be set to None in which case it is
    /// the default configuration or from somewhere else like stdin
    pub path: Option<PathBuf>,
    /// Struct representing the values that can be in the config
    pub config: SvConfig,
}

/// Holds the keys and type of value that can be used on the
/// config file itself
#[derive(Debug, Deserialize, Clone)]
pub struct SvConfig {
    /// Path where the real services are.
    pub svdir: PathBuf, // Path to where directories live
    /// Path where the services can be linked to show they are activated
    pub lndir: PathBuf, // Path to where directories are linked to
}

/// Implements default values for upstream configuration, distributions should
/// ship with their own configuration file in /usr/share/svctrl/config.toml
impl Default for SvConfig {
    fn default() -> Self {
        Self {
            svdir: PathBuf::from("/etc/sv"),
            lndir: PathBuf::from("/var/service"),
        }
    }
}

/// Implements default values for Config with the path value as None as
/// it is not from any file but upstream defaults.
impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            config: SvConfig::default(),
        }
    }
}

/// Returns an Option holding a PathBuf which is where the config was located
///
/// The function searches for the config in 3 system locations suffixed with svctrl/config.toml:
/// - /run for temporary system configuration, /run is usually a tmpfs
/// - /etc for local administrator configuration
///
/// # Examples
///
/// ```
/// if let Some(c) = find() {
///     println!("Found config on {}!", c);
/// }
/// ```
pub fn find() -> Option<PathBuf> {
    let paths = vec![
        Path::new("/run/svctrl/config.toml"),
        Path::new("/etc/svctrl/config.toml"),
    ];

    for path in paths.iter() {
        if path.is_file() {
            return Some(path.to_path_buf());
        }
    }
    None
}

impl Config {
    /// Deserializes a TOML config for svctrl and returns a Config struct with the values given
    ///
    /// # Example
    ///
    /// ```
    /// let conf = Config {
    ///     path: PathBuf::new("/etc/svctrl/config.toml"),
    ///     config: Default::default(),
    /// }
    ///
    /// match conf.open() {
    ///     Ok(_) => (),
    ///     Err(e) => Err(e),
    /// }
    ///
    /// println!("{}", conf);
    /// ```
    pub(crate) fn open(&mut self) -> Result<&mut Self, Error> {
        let mut config_file = std::fs::OpenOptions::new()
            .read(true)
            .open(&self.path.as_ref().unwrap())?;
        let mut config_string = String::new();

        config_file.read_to_string(&mut config_string)?;

        let config_toml: SvConfig = toml::from_str(&config_string)?;

        self.config = config_toml;

        Ok(self)
    }
}
