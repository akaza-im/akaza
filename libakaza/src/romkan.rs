use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;

use anyhow::Context;
use log::info;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

// TODO libkkc みたいにマッピングの継承機能とかあっても良さそう。
// 継承機能作る場合は、"extends" とかをキーワードにしたい。
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RomKanConfig {
    mapping: HashMap<String, String>,
}

fn load_romkan_map(name: &str) -> anyhow::Result<HashMap<String, String>> {
    let pathstr: String = if cfg!(test) {
        format!("{}/../romkan/{}.yml", env!("CARGO_MANIFEST_DIR"), name)
    } else if let Ok(env) = env::var("AKAZA_ROMKAN_DIR") {
        format!("{}/{}.yml", env, name)
    } else {
        let pathbuf = xdg::BaseDirectories::with_prefix("akaza")
            .with_context(|| "Opening xdg directory with 'akaza' prefix")?
            .get_config_file(format!("romkan/{}.yml", name));
        pathbuf.to_string_lossy().to_string()
    };
    info!("Load {}", pathstr);
    let got: RomKanConfig = serde_yaml::from_reader(BufReader::new(
        File::open(&pathstr).with_context(|| pathstr)?,
    ))?;
    Ok(got.mapping)
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
        Self::new("default")
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
}
