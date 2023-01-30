use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result};
use encoding_rs::Encoding;
use log::info;
use regex::Regex;

use crate::dict::merge_dict::merge_dict;
use crate::dict::skk::ari2nasi::Ari2Nasi;

enum ParserState {
    OkuriAri,
    OkuriNasi,
}

pub fn read_skkdict(
    path: &Path,
    encoding: &'static Encoding,
) -> Result<HashMap<String, Vec<String>>> {
    let file = File::open(path).with_context(|| path.to_string_lossy().to_string())?;
    let mut buf: Vec<u8> = Vec::new();
    BufReader::new(file).read_to_end(&mut buf)?;
    let (decoded, _, _) = encoding.decode(buf.as_slice());
    let decoded = decoded.to_string();
    parse_skkdict(decoded.as_str())
}

/**
 * SKK 辞書をパースします。
 */
pub fn parse_skkdict(src: &str) -> Result<HashMap<String, Vec<String>>> {
    let mut ari: HashMap<String, Vec<String>> = HashMap::new();
    let mut nasi: HashMap<String, Vec<String>> = HashMap::new();
    let mut target = &mut ari;

    let comment_regex = Regex::new(";.*")?;

    for line in src.lines() {
        if line.starts_with(";;") {
            if line.contains(";; okuri-ari entries.") {
                target = &mut ari;
                continue;
            } else if line.contains(";; okuri-nasi entries.") {
                target = &mut nasi;
                continue;
            } else {
                // skip comment
                continue;
            }
        }
        if line.is_empty() {
            // skip empty line
            continue;
        }

        let Some((yomi, surfaces)) = line.split_once(' ') else {
            info!("Invalid line: {}", line);
            continue;
        };

        // 読み仮名がアルファベットのものは除外する。
        // `kk /株式会社/` のようなエントリーがライブコンバージョン時に邪魔になるため。
        // https://github.com/akaza-im/akaza/issues/260
        if let Some(first_yomi_char) = yomi.chars().next() {
            if first_yomi_char.is_ascii_alphabetic() {
                continue;
            }
        }

        // example:
        // とくひろ /徳宏/徳大/徳寛/督弘/
        // 末尾の slash が抜けてる場合もあるエントリーが SKK-JISYO.L に入っていたりするので注意。
        let surfaces: Vec<String> = surfaces
            .trim_start_matches('/')
            .trim_end_matches('/')
            .split('/')
            .map(|s| comment_regex.replace(s, "").to_string())
            .filter(|it| !it.is_empty())
            .collect();
        assert!(!yomi.is_empty(), "yomi must not empty: line={line}");
        target.insert(yomi.to_string(), surfaces);
    }

    let ari2nasi = Ari2Nasi::default();
    let ari = ari2nasi.ari2nasi(&ari)?;
    Ok(merge_dict(vec![ari, nasi]))
}

#[cfg(test)]
mod tests {
    use encoding_rs::EUC_JP;
    use log::warn;

    use super::*;

    #[test]
    fn test_skk_l() -> anyhow::Result<()> {
        let dictpath = Path::new("/usr/share/skk/SKK-JISYO.L");
        if !dictpath.exists() {
            warn!("There's no SKK-JISYO.L... Skip this test case.");
            return Ok(());
        }

        let dict = read_skkdict(dictpath, EUC_JP)?;
        assert!(!dict.is_empty());

        Ok(())
    }

    /// 末尾のスラッシュが落ちていても許容する。
    // sars-cov /severe acute respiratory syndrome coronavirus/SARSコロナウイルス
    #[test]
    fn missing_trailing_slash() -> anyhow::Result<()> {
        let src = ";; okuri-nasi entries.\n\
           こな /粉";
        let dict = parse_skkdict(src)?;
        assert_eq!(*dict.get("こな").unwrap(), vec!["粉".to_string()]);

        Ok(())
    }

    /// パース結果が空になる場合は無視する
    #[test]
    fn empty() -> anyhow::Result<()> {
        let src = ";; okuri-nasi entries.\n\
            せみころん /; [Semicolon]/\n\
            お /尾/\n";
        let dict = parse_skkdict(src)?;
        assert_eq!(*dict.get("せみころん").unwrap(), Vec::<String>::new());
        assert_eq!(*dict.get("お").unwrap(), vec!["尾".to_string()]);

        Ok(())
    }

    /// パース結果が空になる場合は無視する
    #[test]
    fn kk() -> anyhow::Result<()> {
        let src = ";; okuri-nasi entries.\n\
            kk /株式会社/\n";
        let dict = parse_skkdict(src)?;
        assert_eq!(dict.get("kk"), None);

        Ok(())
    }
}
