use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result};
use encoding_rs::Encoding;
use log::info;
use regex::Regex;

use crate::skk::ari2nasi::Ari2Nasi;
use crate::skk::merge_skkdict::merge_skkdict;

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
        assert!(!yomi.is_empty(), "yomi must not empty: line={}", line);
        target.insert(yomi.to_string(), surfaces);
    }

    let ari2nasi = Ari2Nasi::default();
    let ari = ari2nasi.ari2nasi(&ari)?;
    Ok(merge_skkdict(vec![ari, nasi]))
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufReader, Read};

    use anyhow::Context;
    use encoding_rs::EUC_JP;

    use super::*;

    #[test]
    fn test_skkdict() -> anyhow::Result<()> {
        let dictpath =
            env!("CARGO_MANIFEST_DIR").to_string() + "/../akaza-data/dict/SKK-JISYO.akaza";
        let file = File::open(&dictpath).with_context(|| format!("path={}", &dictpath))?;
        let mut buf = String::new();
        BufReader::new(file).read_to_string(&mut buf)?;
        let dict = parse_skkdict(buf.as_str())?;
        assert_eq!(*dict.get("ぶかわ").unwrap(), vec!["武川".to_string()]);

        Ok(())
    }

    #[test]
    fn test_skk_l() -> anyhow::Result<()> {
        let dictpath =
            env!("CARGO_MANIFEST_DIR").to_string() + "/../akaza-data/skk-dev-dict/SKK-JISYO.L";
        let dict = read_skkdict(Path::new(dictpath.as_str()), EUC_JP)?;
        assert!(!dict.is_empty());

        Ok(())
    }

    /// 末尾のスラッシュが落ちていても許容する。
    // sars-cov /severe acute respiratory syndrome coronavirus/SARSコロナウイルス
    #[test]
    fn missing_trailing_slash() -> anyhow::Result<()> {
        let src = ";; okuri-nasi entries.\n\
            sars-cov /severe acute respiratory syndrome coronavirus/SARSコロナウイルス";
        let dict = parse_skkdict(src)?;
        assert_eq!(
            *dict.get("sars-cov").unwrap(),
            vec![
                "severe acute respiratory syndrome coronavirus".to_string(),
                "SARSコロナウイルス".to_string(),
            ]
        );

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
}
