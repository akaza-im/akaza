/*
---
dicts:
  - path: /usr/share/skk/SKK-JISYO.okinawa
    encoding: euc-jp
    dict_type: skk
 */
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;

use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use DictEncoding::Utf8;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    pub dicts: Vec<DictConfig>,
    pub single_term: Vec<DictConfig>,

    /// ローマ字かな変換テーブルの指定
    /// "default", "kana", etc.
    #[serde(default = "default_romkan")]
    pub romkan: String,

    /// キーマップテーブルの指定
    /// "default", "atok", etc.
    #[serde(default = "default_keymap")]
    pub keymap: String,

    /// Model の指定
    /// "default", etc.
    #[serde(default = "default_model")]
    pub model: String,
}

fn default_romkan() -> String {
    "default".to_string()
}

fn default_keymap() -> String {
    "default".to_string()
}

fn default_model() -> String {
    "default".to_string()
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self> {
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct DictConfig {
    pub path: String,

    /// Encoding of the dictionary
    /// Default: UTF-8
    // #[serde(default = "default_encoding")]
    pub encoding: DictEncoding,

    // #[serde(default = "default_dict_type")]
    pub dict_type: DictType,
}

fn default_encoding() -> DictEncoding {
    Utf8
}

fn default_dict_type() -> DictType {
    DictType::SKK
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum DictEncoding {
    EucJp,
    Utf8,
}

impl Default for DictEncoding {
    fn default() -> Self {
        Utf8
    }
}

impl Display for DictEncoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl DictEncoding {
    pub fn as_str(&self) -> &'static str {
        match self {
            DictEncoding::Utf8 => "UTF-8",
            DictEncoding::EucJp => "EUC-JP",
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum DictType {
    SKK,
}

impl Display for DictType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for DictType {
    fn default() -> Self {
        DictType::SKK
    }
}

impl DictType {
    pub fn as_str(&self) -> &'static str {
        match self {
            &DictType::SKK => "SKK",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() -> anyhow::Result<()> {
        let config = Config::load_from_file("../config.sample.yml")?;
        assert_eq!(config.dicts.len(), 2);
        assert_eq!(
            config.dicts[0],
            DictConfig {
                path: "/usr/share/skk/SKK-JISYO.L".to_string(),
                encoding: DictEncoding::EucJp,
                dict_type: DictType::SKK,
            }
        );
        Ok(())
    }
}
