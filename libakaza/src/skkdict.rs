use std::collections::HashMap;

use log::info;

type SkkDictParsedData = HashMap<String, Vec<String>>;

enum ParserState {
    OkuriAri,
    OkuriNasi,
}

/**
 * SKK 辞書をパースします。
 */
fn parse_skkdict(src: &str) -> anyhow::Result<(SkkDictParsedData, SkkDictParsedData)> {
    let mut ari: SkkDictParsedData = HashMap::new();
    let mut nasi: SkkDictParsedData = HashMap::new();
    let mut target = &mut ari;

    for line in src.lines() {
        let line: &str = line.trim();
        if line.contains(";; okuri-ari entries.") {
            target = &mut ari;
            continue;
        }
        if line.contains(";; okuri-nasi entries.") {
            target = &mut nasi;
            continue;
        }
        if line.contains(";;") {
            // skip comment
            continue;
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
        let surfaces: Vec<String> = surfaces[1..surfaces.len() - 1]
            .split('/')
            .map(|s| s.to_string())
            .collect();
        target.insert(yomi.to_string(), surfaces);
    }

    Ok((ari, nasi))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use std::fs::File;
    use std::io::{BufReader, Read};

    use super::*;

    #[test]
    fn test_skkdict() -> anyhow::Result<()> {
        let dictpath =
            env!("CARGO_MANIFEST_DIR").to_string() + "/../akaza-data/dict/SKK-JISYO.akaza";
        let file = File::open(&dictpath).with_context(|| format!("path={}", &dictpath))?;
        let mut buf = String::new();
        BufReader::new(file).read_to_string(&mut buf)?;
        let (_, nasi) = parse_skkdict(buf.as_str())?;
        assert_eq!(*nasi.get("ぶかわ").unwrap(), vec!["武川".to_string()]);

        Ok(())
    }
}
