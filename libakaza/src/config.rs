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
use std::io::{BufReader, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};

use DictEncoding::Utf8;

use crate::config::DictUsage::{Normal, SingleTerm};
use crate::resource::detect_resource_path;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    pub dicts: Vec<DictConfig>,

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
    detect_resource_path("romkan", "default").unwrap()
}

fn default_keymap() -> String {
    detect_resource_path("keymap", "default").unwrap()
}

fn default_model() -> String {
    detect_resource_path("model", "default").unwrap()
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    pub fn file_name() -> Result<PathBuf> {
        let basedir = xdg::BaseDirectories::with_prefix("akaza")?;
        Ok(basedir.get_config_file("config.yml"))
    }

    pub fn save(&self) -> Result<()> {
        let file_name = Self::file_name()?;
        let yml = serde_yaml::to_string(self)?;
        info!("Write to file: {}", file_name.to_str().unwrap());
        let mut fp = File::create(file_name)?;
        fp.write_all(yml.as_bytes())?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let configfile = Self::file_name()?;
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
    pub encoding: DictEncoding,

    pub dict_type: DictType,

    pub usage: DictUsage,
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
            Utf8 => "UTF-8",
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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum DictUsage {
    Normal,
    SingleTerm,
    Disabled,
}

impl Default for DictUsage {
    fn default() -> Self {
        Normal
    }
}

impl DictUsage {
    pub fn from(s: &str) -> Result<DictUsage> {
        match s {
            "Normal" => Ok(Normal),
            "SingleTerm" => Ok(SingleTerm),
            "Disabled" => Ok(DictUsage::Disabled),
            _ => bail!("Unknown name: {:?}", s),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Normal => "Normal",
            SingleTerm => "SingleTerm",
            DictUsage::Disabled => "Disabled",
        }
    }

    pub fn text_jp(&self) -> &'static str {
        match self {
            Normal => "通常辞書",
            SingleTerm => "単項",
            DictUsage::Disabled => "無効",
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
                usage: DictUsage::Normal,
            }
        );
        Ok(())
    }
}
