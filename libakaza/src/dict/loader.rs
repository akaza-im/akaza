use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Result;
use anyhow::{bail, Context};
use encoding_rs::{EUC_JP, UTF_8};
use log::{error, info};

use marisa_sys::{Keyset, Marisa};

use crate::config::DictConfig;
use crate::dict::merge_dict::merge_dict;
use crate::dict::skk::read::read_skkdict;
use crate::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;

fn try_get_mtime(path: &str) -> Result<u64> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let mtime = metadata.modified()?;
    let t = mtime.duration_since(SystemTime::UNIX_EPOCH)?;
    Ok(t.as_secs())
}

pub fn load_dicts_ex(
    dict_configs: &Vec<DictConfig>,
    cache_name: &str,
) -> Result<MarisaKanaKanjiDict> {
    // さて、ここで、全部の依存先ファイルの mtime の max とキャッシュファイルの mtime の max を比較する
    // 更新が必要だったら、更新する。
    let max_dict_mtime = dict_configs
        .iter()
        .map(|it| try_get_mtime(&it.path).unwrap_or_else(|_| 0_u64))
        .max()
        .unwrap_or_else(|| 0_u64);

    // cache file のパスを得る

    let base_dirs = xdg::BaseDirectories::with_prefix("akaza")
        .with_context(|| "xdg directory with 'akaza' prefix")?;
    let cache_path = base_dirs.get_cache_file(cache_name);
    let cache_mtime =
        try_get_mtime(cache_path.to_string_lossy().to_string().as_str()).unwrap_or_else(|_| 0_u64);

    if cache_mtime >= max_dict_mtime {
        info!(
            "Cache is not fresh! {:?} => {}",
            dict_configs,
            cache_path.to_string_lossy()
        );
        Ok(MarisaKanaKanjiDict::load(
            cache_path.to_string_lossy().to_string().as_str(),
        )?)
    } else {
        info!(
            "Cache is not fresh! {:?} => {}",
            dict_configs,
            cache_path.to_string_lossy()
        );
        let dicts = load_dicts(dict_configs)?;
        let mut keyset = Keyset::default();
        for (kana, surfaces) in dicts {
            keyset.push_back(
                [
                    kana.as_bytes(),
                    b"\xff", // seperator
                    surfaces.join("/").as_bytes(),
                ]
                .concat()
                .as_slice(),
            );
        }

        let mut marisa = Marisa::default();
        marisa.build(&keyset);

        // キャッシュを更新する必要あり。
        Ok(MarisaKanaKanjiDict::new(marisa))
    }
}

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
