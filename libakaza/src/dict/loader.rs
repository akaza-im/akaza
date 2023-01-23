use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Result;
use anyhow::{bail, Context};
use encoding_rs::{EUC_JP, UTF_8};
use log::{error, info};

use crate::config::DictConfig;
use crate::dict::merge_dict::merge_dict;
use crate::dict::skk::read::read_skkdict;
use crate::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;

fn try_get_mtime(path: &str) -> Result<u128> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let mtime = metadata.modified()?;
    let t = mtime.duration_since(SystemTime::UNIX_EPOCH)?;
    Ok(t.as_millis())
}

/// - `dict_configs`: 辞書の読み込み設定
/// - `cache_name`: キャッシュファイル名。 `~/.cache/akaza/kana_kanji_cache.marisa` とかにでる。
pub fn load_dicts_ex(
    dict_configs: &Vec<DictConfig>,
    cache_name: &str,
) -> Result<MarisaKanaKanjiDict> {
    // さて、ここで、全部の依存先ファイルの mtime の max とキャッシュファイルの mtime の max を比較する
    // 更新が必要だったら、更新する。
    let p = dict_configs
        .iter()
        .map(|it| try_get_mtime(&it.path).unwrap_or(0_u128))
        .collect::<Vec<_>>();
    info!("mtimes: {:?}", p);

    let max_dict_mtime = dict_configs
        .iter()
        .map(|it| try_get_mtime(&it.path).unwrap_or(0_u128))
        .max()
        .unwrap_or(0_u128);

    // cache file のパスを得る
    let base_dirs = xdg::BaseDirectories::with_prefix("akaza")
        .with_context(|| "xdg directory with 'akaza' prefix")?;
    base_dirs.create_cache_directory("")?;
    let cache_path = base_dirs
        .get_cache_file(cache_name)
        .to_string_lossy()
        .to_string();
    let cache_mtime = try_get_mtime(&cache_path).unwrap_or(0_u128);

    // 現在の Config をシリアライズする。
    let config_serialized = serde_yaml::to_string(dict_configs)?;
    info!("SERIALIZED: {:?}", config_serialized);

    if cache_mtime >= max_dict_mtime {
        match MarisaKanaKanjiDict::load(cache_path.as_str()) {
            Ok(dict) => {
                let dict_serialized = dict.cache_serialized();
                if dict_serialized == config_serialized {
                    // キャッシュファイルを書いた時の設定と同じかどうかを確認する
                    // 設定が違う場合は、キャッシュを作り直す必要がある。
                    info!("Cache is fresh! {:?} => {}", dict_configs, cache_path);
                    return Ok(dict);
                } else {
                    info!(
                        "DictConfig was modified...: {:?}!={:?}",
                        dict_serialized, config_serialized
                    );
                }
            }
            Err(err) => {
                info!("Cannot load {:?}: {:?}", cache_path, err)
            }
        }
    }

    info!("Cache is not fresh! {:?} => {}", dict_configs, cache_path);
    let dicts = load_dicts(dict_configs)?;

    MarisaKanaKanjiDict::build(dicts, &cache_path, &config_serialized)
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::io::Write;
    use std::{env, thread, time};

    use anyhow::Result;
    use log::LevelFilter;
    use tempfile::{tempdir, NamedTempFile};

    use super::*;

    #[test]
    fn test_load_dict_ex() -> Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .is_test(true)
            .try_init();

        let dictfile = NamedTempFile::new().unwrap();

        let cachedir = tempdir()?;
        info!("tmpdir: {}", cachedir.path().to_str().unwrap());
        env::set_var("XDG_CACHE_HOME", cachedir.path().to_str().unwrap());

        {
            let mut fp = File::create(dictfile.path())?;
            fp.write_all(
                ";; okuri-ari entries.\n\
            ;; okuri-nasi entries.\n\
            たこ /凧/\n"
                    .as_bytes(),
            )?;
        }

        let loaded = load_dicts_ex(
            &vec![DictConfig {
                path: dictfile.path().to_str().unwrap().to_string(),
                encoding: None,
                dict_type: "skk".to_string(),
            }],
            "test",
        )?;
        assert_eq!(loaded.yomis(), vec!["たこ"]);

        // timestamp がずれるように 10msec 休む
        thread::sleep(time::Duration::from_millis(10));

        {
            let mut fp = File::create(dictfile.path())?;
            fp.write_all(
                ";; okuri-ari entries.\n\
            ;; okuri-nasi entries.\n\
            たこ /凧/\n\
            いか /烏賊/\n"
                    .as_bytes(),
            )?;
        }

        // ファイルを書き直したら、キャッシュも読みなおしてほしい。
        let loaded = load_dicts_ex(
            &vec![DictConfig {
                path: dictfile.path().to_str().unwrap().to_string(),
                encoding: None,
                dict_type: "skk".to_string(),
            }],
            "test",
        )?;
        assert_eq!(
            loaded
                .yomis()
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>(),
            HashSet::from(["いか".to_string(), "たこ".to_string()])
        );

        let loaded = load_dicts_ex(
            &vec![DictConfig {
                path: dictfile.path().to_str().unwrap().to_string(),
                encoding: None,
                dict_type: "skk".to_string(),
            }],
            "test",
        )?;
        assert_eq!(
            loaded
                .yomis()
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>(),
            HashSet::from(["いか".to_string(), "たこ".to_string()])
        );

        Ok(())
    }

    /// 設定ファイルが書き換えられたら読み直す。
    /// 書き換えられたら読み直す。
    #[test]
    fn test_if_config_was_changed() -> Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .is_test(true)
            .try_init();

        let dict1 = NamedTempFile::new().unwrap();
        let dict2 = NamedTempFile::new().unwrap();

        let cachedir = tempdir()?;
        info!("tmpdir: {}", cachedir.path().to_str().unwrap());
        env::set_var("XDG_CACHE_HOME", cachedir.path().to_str().unwrap());

        {
            let mut fp = File::create(dict1.path())?;
            fp.write_all(
                ";; okuri-ari entries.\n\
            ;; okuri-nasi entries.\n\
            たこ /凧/\n"
                    .as_bytes(),
            )?;
        }

        {
            let mut fp = File::create(dict2.path())?;
            fp.write_all(
                ";; okuri-ari entries.\n\
            ;; okuri-nasi entries.\n\
            えび /海老/\n"
                    .as_bytes(),
            )?;
        }

        // dict1 のみを読んでみる。
        let loaded = load_dicts_ex(
            &vec![DictConfig {
                path: dict1.path().to_str().unwrap().to_string(),
                encoding: None,
                dict_type: "skk".to_string(),
            }],
            "test",
        )?;
        assert_eq!(loaded.yomis(), vec!["たこ"]);

        // dict2 も指定するパターン。
        let loaded = load_dicts_ex(
            &vec![
                DictConfig {
                    path: dict1.path().to_str().unwrap().to_string(),
                    encoding: None,
                    dict_type: "skk".to_string(),
                },
                DictConfig {
                    path: dict2.path().to_str().unwrap().to_string(),
                    encoding: None,
                    dict_type: "skk".to_string(),
                },
            ],
            "test",
        )?;
        assert_eq!(
            loaded
                .yomis()
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>(),
            HashSet::from(["たこ".to_string(), "えび".to_string()])
        );

        Ok(())
    }
}
