use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

use anyhow::bail;
use anyhow::Result;
use encoding_rs::{EUC_JP, UTF_8};
use log::{error, info};

use crate::config::DictConfig;
use crate::dict::merge_dict::merge_dict;
use crate::dict::skk::read::read_skkdict;

pub fn load_dicts(dict_configs: &Vec<DictConfig>) -> Result<HashMap<String, Vec<String>>> {
    let mut dicts: Vec<HashMap<String, Vec<String>>> = Vec::new();
    for dict_config in dict_configs {
        match load_dict(dict_config) {
            Ok(dict) => {
                // TODO 辞書をうまく使う
                dicts.push(dict);
            }
            Err(err) => {
                error!("Cannot load dictionary: {:?}. {}", dict_config, err);
                // 一顧の辞書の読み込みに失敗しても、他の辞書は読み込むべきなので
                // 処理は続行する
            }
        }
    }
    Ok(merge_dict(dicts))
}

pub fn load_dict(dict: &DictConfig) -> Result<HashMap<String, Vec<String>>> {
    // TODO キャッシュ機構を入れる。
    info!(
        "Loading dictionary: {} {:?} {}",
        dict.path, dict.encoding, dict.dict_type
    );
    let encoding = match &dict.encoding {
        Some(encoding) => match encoding.to_ascii_lowercase().as_str() {
            "euc-jp" | "euc_jp" => EUC_JP,
            "utf-8" => UTF_8,
            _ => {
                bail!(
                    "Unknown enconding in configuration: {} for {}",
                    encoding,
                    dict.path
                )
            }
        },
        None => UTF_8,
    };

    match dict.dict_type.as_str() {
        "skk" => {
            let t1 = SystemTime::now();
            let merged = read_skkdict(Path::new(dict.path.as_str()), encoding)?;
            let t2 = SystemTime::now();
            info!(
                "Loaded {}: {} entries in {} msec",
                dict.path,
                merged.len(),
                t2.duration_since(t1).unwrap().as_millis()
            );
            Ok(merged)
        }
        _ => {
            bail!(
                "Unknown dictionary type: {} for {}",
                dict.dict_type,
                dict.path
            );
        }
    }
}
