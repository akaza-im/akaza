/*
---
dicts:
  - path: /usr/share/skk/SKK-JISYO.okinawa
    encoding: euc-jp
    dict_type: skk
 */
use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    pub dicts: Vec<DictConfig>,
    pub single_term: Option<Vec<DictConfig>>,
}

impl Config {
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    pub fn load() -> Result<Self> {
        let basedir = xdg::BaseDirectories::with_prefix("akaza")?;
        let configfile = basedir.get_config_file("config.yml");
        let config = match Config::load_from_file(configfile.to_str().unwrap()) {
            Ok(config) => config,
            Err(err) => {
                warn!(
                    "Cannot load configuration file: {} {}",
                    configfile.to_string_lossy(),
                    err
                );
                return Ok(Config::default());
            }
        };
        info!(
            "Loaded config file: {}, {:?}",
            configfile.to_string_lossy(),
            config
        );
        Ok(config)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DictConfig {
    pub path: String,
    /// Default: UTF-8
    pub encoding: Option<String>,
    pub dict_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() -> anyhow::Result<()> {
        let config = Config::load_from_file("../config.sample.yml")?;
        assert_eq!(config.dicts.len(), 1);
        assert_eq!(
            config.dicts[0],
            DictConfig {
                path: "/usr/share/skk/SKK-JISYO.okinawa".to_string(),
                encoding: Some("euc-jp".to_string()),
                dict_type: "skk".to_string()
            }
        );
        Ok(())
    }
}
