use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const CONFIG_KEYS: [&str; 4] = [
    "enabled",
    "reverse_mouse",
    "reverse_trackpad",
    "reverse_horizontal",
];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub reverse_mouse: bool,
    pub reverse_trackpad: bool,
    pub reverse_horizontal: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            reverse_mouse: true,
            reverse_trackpad: false,
            reverse_horizontal: false,
        }
    }
}

impl Config {
    pub fn path() -> Result<PathBuf, String> {
        let home = env::var_os("HOME").ok_or_else(|| "HOME is not set".to_owned())?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("ScrollSplit")
            .join("config.toml"))
    }

    pub fn load_or_create() -> Result<Self, String> {
        let path = Self::path()?;
        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents)
                .map_err(|error| format!("cannot parse {}: {error}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let config = Self::default();
                config.save()?;
                Ok(config)
            }
            Err(error) => Err(format!("cannot read {}: {error}", path.display())),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path()?;
        let parent = path
            .parent()
            .ok_or_else(|| format!("invalid config path: {}", path.display()))?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        fs::write(&path, self.to_toml()?)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))
    }

    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|error| format!("cannot serialize config: {error}"))
    }

    pub fn set(&mut self, key: &str, value: bool) -> Result<(), String> {
        match key {
            "enabled" => self.enabled = value,
            "reverse_mouse" => self.reverse_mouse = value,
            "reverse_trackpad" => self.reverse_trackpad = value,
            "reverse_horizontal" => self.reverse_horizontal = value,
            _ => {
                return Err(format!(
                    "unknown key {key:?}; expected one of {}",
                    CONFIG_KEYS.join(", ")
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn default_config_matches_documented_values() {
        let config = Config::default();
        assert!(config.enabled);
        assert!(config.reverse_mouse);
        assert!(!config.reverse_trackpad);
        assert!(!config.reverse_horizontal);
    }

    #[test]
    fn config_round_trips_through_toml() {
        let config = Config::default();
        let encoded = config.to_toml().unwrap();
        let decoded: Config = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded, config);
    }
}
