use crate::errors::Error;
use serde::Deserialize;
use std::fmt::Result as fmtResult;
use std::fmt::{Display, Formatter};
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

/// `fmt::Display` for Config, showing in the TOML format the configuration is written in
impl Display for Config {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        if self.path.is_some() {
            writeln!(f, "path = '{}'", self.path.as_ref().unwrap().display());
        }
        writeln!(f, "svdir = '{}'", self.svdir.display());
        write!(f, "lndir = '{}'", self.lndir.display())
    }
}

/// Holds the location of a config as path and all the values that can be used
/// in the config
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Path to where the config is located for opening, reading and writing
    /// it is an option because it can be set to None in which case it is
    /// the default configuration or from somewhere else like stdin. This
    /// is the only field that isn't in the config
    #[serde(skip_serializing)]
    pub path: Option<PathBuf>,
    /// Path where the real services are.
    pub svdir: PathBuf, // Path to where directories live
    /// Path where the services can be linked to show they are activated
    pub lndir: PathBuf, // Path to where directories are linked to
}

/// Implements default values for upstream configuration, distributions should
/// ship with their own configuration file in /usr/share/svctrl/config.toml
impl Default for Config {
    fn default() -> Self {
        Self {
            path: None,
            svdir: PathBuf::from("/etc/sv"),
            lndir: PathBuf::from("/var/service"),
        }
    }
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
    fn open(&mut self) -> Result<&mut Self, Error> {
        let path = match self.path.as_ref() {
            Some(p) => p,
            None => return Err(Error::ConfNone),
        };

        let mut config_file = std::fs::OpenOptions::new().read(true).open(path)?;
        let mut config_string = String::new();

        config_file.read_to_string(&mut config_string)?;

        let config_toml: Self = toml::from_str(&config_string)?;

        self.svdir = config_toml.svdir;
        self.lndir = config_toml.lndir;

        Ok(self)
    }

    /// Impleentation of new for Config, uses the default values
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Returns an Option holding a `PathBuf` which is where the config was located or none if none
    /// is found
    ///
    /// The function searches for the config in 3 system locations suffixed with svctrl/config.toml:
    /// - /run for temporary system configuration, /run is usually a tmpfs
    /// - /etc for local administrator configuration
    /// - /usr/share for configuration from the distro
    ///
    /// # Examples
    ///
    /// ```
    /// if let Some(c) = find() {
    ///     println!("Found config on {}!", c);
    /// }
    /// ```
    fn find_conf(&mut self) {
        let paths = vec![
            Path::new("/run/svctrl/config.toml"),
            Path::new("/etc/svctrl/config.toml"),
            Path::new("/usr/share/svctrl/config.toml"),
        ];

        for path in &paths {
            if path.is_file() {
                self.path = Some(path.to_path_buf());
                return;
            }
        }
        self.path = None;
    }

    /// Takes a configuration::Config struct and tries to load configuration from .path
    /// with .open()
    fn load_conf(&mut self) -> Result<&mut Self, Error> {
        if self.path.is_none() {
            return Err(Error::CalledWithoutConf);
        }

        // Store this temporary variable that is the path to be used
        // in the Eror in case self.open() fails
        let path_for_err = self.path.clone().unwrap();

        match self.open() {
            Ok(e) => Ok(e),
            Err(e) => return Err(Error::FailedToLoadConf(path_for_err, e.to_string())),
        }
    }

    /// Takes a configuration::Config struct and tries to find a configuration with .find_path()
    /// unless a value was given for configuration, then use .open() to load the configuration
    /// and then return it.
    ///
    /// If .find_conf() returns None for the .path value then we hope
    pub(crate) fn set_conf(&mut self, conf_path: Option<PathBuf>) -> Result<&mut Self, Error> {
        // Try to find path, if conf_path isn't None then it will be used instead
        // and returned with success, no checks on whether the file exists which
        // doesn't matter because it will be caught by self.load_conf()
        match conf_path {
            Some(conf_path) => self.path = Some(conf_path),
            None => self.find_conf(),
        }

        // Tries to load configuration from .path field of the given struct it is
        // guaranteed to be set by calling find_path before it, it will most
        // likely error out when the user passes a configuration PathBuf via conf_path
        // that doesn't actually exist
        //
        // In the case of svctrl the user can pass '-c' '--config' to an inexistant
        // file.
        if self.path.is_some() {
            match self.load_conf() {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(self)
    }
}
