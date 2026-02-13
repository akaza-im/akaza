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

use anyhow::{bail, Context, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};

use DictEncoding::Utf8;

use crate::config::DictUsage::{Normal, SingleTerm};
use crate::graph::reranking::ReRankingWeights;
use crate::resource::detect_resource_path;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Config {
    /// ローマ字かな変換テーブルの指定
    /// "default", "kana", etc.
    #[serde(default = "default_romkan")]
    pub romkan: String,

    /// キーマップテーブルの指定
    /// "default", "atok", etc.
    #[serde(default = "default_keymap")]
    pub keymap: String,

    #[serde(default = "default_engine_config")]
    pub engine: EngineConfig,

    /// ライブ変換
    #[serde(default = "default_live_conversion")]
    pub live_conversion: bool,
}

fn default_romkan() -> String {
    detect_resource_path("romkan", "default.yml").unwrap()
}

fn default_keymap() -> String {
    detect_resource_path("keymap", "default.yml").unwrap()
}

fn default_engine_config() -> EngineConfig {
    EngineConfig {
        dicts: find_default_dicts(),
        dict_cache: true,
        model: default_model(),
        reranking_weights: ReRankingWeights::default(),
    }
}

fn find_default_dicts() -> Vec<DictConfig> {
    let mut dicts: Vec<DictConfig> = Vec::new();

    if let Ok(dir) = xdg::BaseDirectories::with_prefix("skk") {
        if let Some(file) = dir.find_data_files("SKK-JISYO.L").next() {
            dicts.push(DictConfig {
                path: file.to_string_lossy().to_string(),
                encoding: DictEncoding::EucJp,
                dict_type: DictType::SKK,
                usage: Normal,
            });
        }
    }

    info!("default dictionaries: {:?}", dicts);

    dicts
}

fn default_live_conversion() -> bool {
    false
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
        if let Some(parent) = file_name.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Cannot create config dir: {}", parent.display()))?;
        }
        info!("Write to file: {}", file_name.display());
        let mut fp = File::create(&file_name)
            .with_context(|| format!("Cannot open config file: {}", file_name.display()))?;
        fp.write_all(yml.as_bytes())
            .with_context(|| format!("Cannot write config file: {}", file_name.display()))?;
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
                let config: Config = serde_yaml::from_str("").unwrap();
                info!("Loaded default configuration: {:?}", config);
                return Ok(config);
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
pub struct EngineConfig {
    pub dicts: Vec<DictConfig>,

    /// 辞書のキャッシュ機能のオンオフ設定
    #[serde(default = "default_dict_cache")]
    pub dict_cache: bool,

    /// Model の指定
    /// "default", etc.
    #[serde(default = "default_model")]
    pub model: String,

    /// リランキング重み（省略時はデフォルト値 = 従来と同じ挙動）
    #[serde(default)]
    pub reranking_weights: ReRankingWeights,
}

fn default_dict_cache() -> bool {
    true
}

fn default_model() -> String {
    detect_resource_path("model", "default").unwrap()
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct DictConfig {
    #[serde(default = "default_path")]
    pub path: String,

    /// Encoding of the dictionary
    /// Default: UTF-8
    #[serde(default = "default_encoding")]
    pub encoding: DictEncoding,

    #[serde(default = "default_dict_type")]
    pub dict_type: DictType,

    #[serde(default = "default_dict_usage")]
    pub usage: DictUsage,
}

fn default_path() -> String {
    "".to_string()
}

fn default_encoding() -> DictEncoding {
    Utf8
}

fn default_dict_type() -> DictType {
    DictType::SKK
}

fn default_dict_usage() -> DictUsage {
    DictUsage::Disabled
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum DictEncoding {
    EucJp,
    #[default]
    Utf8,
}

impl Display for DictEncoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl DictEncoding {
    pub fn from(s: &str) -> Result<DictEncoding> {
        match s {
            "EUC-JP" | "EucJp" => Ok(DictEncoding::EucJp),
            "UTF-8" | "Utf8" => Ok(DictEncoding::Utf8),
            _ => bail!("Unknown encoding: {:?}", s),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Utf8 => "UTF-8",
            DictEncoding::EucJp => "EUC-JP",
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum DictType {
    #[default]
    SKK,
}

impl Display for DictType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl DictType {
    pub fn as_str(&self) -> &'static str {
        match self {
            &DictType::SKK => "SKK",
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum DictUsage {
    #[default]
    Normal,
    SingleTerm,
    Disabled,
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
