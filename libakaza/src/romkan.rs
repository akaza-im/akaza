use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use crate::resource::detect_resource_path;
use anyhow::Context;
use log::info;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RomKanConfig {
    mapping: HashMap<String, Option<String>>,
    extends: Option<String>,
}

fn load_romkan_map(file_path: &str) -> anyhow::Result<HashMap<String, String>> {
    info!("Loading romkan map: {}", file_path);
    let got: RomKanConfig = serde_yaml::from_reader(BufReader::new(
        File::open(file_path).with_context(|| file_path.to_string())?,
    ))?;

    if let Some(parent) = got.extends {
        // 継承しているので親を読み込む。
        // 再帰的な処理になる。
        let path = detect_resource_path("romkan", &format!("{}.yml", parent))?;
        let mut parent = load_romkan_map(&path)?;

        for (k, v) in got.mapping {
            if let Some(v) = v {
                parent.insert(k, v);
            } else {
                parent.remove(&k);
            }
        }

        Ok(parent)
    } else {
        // 継承していないのでそのまま。
        Ok(got
            .mapping
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k.clone(), v.clone().unwrap()))
            .collect())
    }
}

pub struct RomKanConverter {
    pub mapping_name: String,
    romkan_pattern: Regex,
    romkan_map: HashMap<String, String>,
    last_char_pattern: Regex,
}

impl RomKanConverter {
    pub fn new(mapping_name: &str) -> anyhow::Result<RomKanConverter> {
        let romkan_map = load_romkan_map(mapping_name)?;

        let mut romas = Vec::from_iter(romkan_map.keys());
        // 長いキーから一致させるようにする。
        romas.sort_by_key(|a| std::cmp::Reverse(a.len()));
        let mut pattern = String::from("(");
        for x in romas {
            pattern += &regex::escape(x);
            pattern += "|";
        }
        pattern += ".)";

        let romkan_pattern = Regex::new(&pattern).unwrap();
        let last_char_pattern = Regex::new(&(pattern + "$")).unwrap();

        Ok(RomKanConverter {
            mapping_name: mapping_name.to_string(),
            romkan_pattern,
            romkan_map,
            last_char_pattern,
        })
    }

    pub fn default_mapping() -> anyhow::Result<RomKanConverter> {
        Self::new(&detect_resource_path("romkan", "default.yml")?)
    }
}

impl RomKanConverter {
    pub fn to_hiragana(&self, src: &str) -> String {
        let src = src.to_ascii_lowercase();
        let src = src.replace("nn", "n'"); // replace nn as n'.
        let retval = self.romkan_pattern.replace_all(&src, |caps: &Captures| {
            let rom = caps.get(1).unwrap().as_str();
            if let Some(e) = self.romkan_map.get(rom) {
                e.to_string()
            } else {
                rom.to_string()
            }
        });
        retval.into_owned()
    }

    pub fn remove_last_char(&self, src: &str) -> String {
        self.last_char_pattern.replace(src, "").to_string()
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::*;

    #[test]
    fn test_to_hiragana_simple() -> anyhow::Result<()> {
        let converter = RomKanConverter::default_mapping()?;
        assert_eq!(converter.to_hiragana("aiu"), "あいう");
        Ok(())
    }

    #[test]
    fn test_to_hiragana() -> anyhow::Result<()> {
        let data = [
            ("a", "あ"),
            ("ba", "ば"),
            ("hi", "ひ"),
            ("wahaha", "わはは"),
            ("thi", "てぃ"),
            ("better", "べってr"),
            ("[", "「"),
            ("]", "」"),
            ("wo", "を"),
            ("du", "づ"),
            ("we", "うぇ"),
            ("di", "ぢ"),
            ("fu", "ふ"),
            ("ti", "ち"),
            ("wi", "うぃ"),
            ("we", "うぇ"),
            ("wo", "を"),
            ("z,", "‥"),
            ("z.", "…"),
            ("z/", "・"),
            ("z[", "『"),
            ("z]", "』"),
            ("du", "づ"),
            ("di", "ぢ"),
            ("fu", "ふ"),
            ("ti", "ち"),
            ("wi", "うぃ"),
            ("we", "うぇ"),
            ("wo", "を"),
            ("sorenawww", "それなwww"),
            ("komitthi", "こみってぃ"),
            ("ddha", "っでゃ"),
            ("zzye", "っじぇ"),
            ("tanni", "たんい"),
        ];
        let converter = RomKanConverter::default_mapping()?;
        for (rom, kana) in data {
            assert_eq!(converter.to_hiragana(rom), kana);
        }
        Ok(())
    }

    #[test]
    fn remove_last_char() -> anyhow::Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .try_init();

        let cases: Vec<(&str, &str)> = vec![
            ("aka", "a"),
            ("sona", "so"),
            ("son", "so"),
            ("sonn", "so"),
            ("sonnna", "sonn"),
            ("sozh", "so"),
        ];
        let romkan = RomKanConverter::default_mapping()?;
        for (src, expected) in cases {
            let got = romkan.remove_last_char(src);
            assert_eq!(got, expected);
        }
        Ok(())
    }

    #[test]
    fn test_atok() -> anyhow::Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .try_init();

        let converter = RomKanConverter::new("../romkan/atok.yml")?;
        assert_eq!(converter.to_hiragana("aiu"), "あいう");
        // zya が null で上書きされて消えてる
        assert_eq!(converter.to_hiragana("zya"), "zや");
        // 追加したぶんが効いてる
        assert_eq!(converter.to_hiragana("tso"), "つぉ");
        Ok(())
    }

    #[test]
    fn test_azik() -> anyhow::Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .try_init();

        let converter = RomKanConverter::new("../romkan/azik.yml")?;
        assert_eq!(converter.to_hiragana("dn"), "だん");
        Ok(())
    }
}
