extern crate toml;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub path: String,
    pub config: SvConfig,
}

#[derive(Deserialize)]
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

impl Config {

    /*
     * Returns a given config
     */
    /*
    fn open(&mut self) -> Result<&mut Self, Error> {
        let mut config_file = std::fs::OpenOptions::new().read(true).open(&self.path)?;
        let mut config_string = String::new();

        config_file.read_to_string(&mut config_string)?;

        let config_toml: svConfig = toml::from_str(&config_string)?;

        self.config = config_toml;

        Ok(self)
    }
    */

}
