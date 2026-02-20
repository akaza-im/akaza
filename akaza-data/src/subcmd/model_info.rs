use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::Result;
use rsmarisa::Trie;

use libakaza::lm::model_metadata::read_metadata_from_trie;

/// モデルファイルのメタデータを表示する。
/// marisa-trie 形式 (.model) とテキスト形式 (SKK-JISYO.*) の両方に対応。
pub fn model_info(path: &str) -> Result<()> {
    println!("File: {}", path);

    // まず marisa-trie として読み込みを試みる
    let mut trie = Trie::new();
    if trie.load(path).is_ok() {
        println!("Type: marisa-trie");
        println!("Keys: {}", trie.num_keys());
        let metadata = read_metadata_from_trie(&trie);
        print!("{}", metadata);
        return Ok(());
    }

    // テキストファイル（SKK辞書）として読み込み
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    println!("Type: skk-dict");

    let mut found = false;
    for line in reader.lines() {
        let line = line?;
        if !line.starts_with(";;") {
            break;
        }
        let comment = line.trim_start_matches(";;").trim();
        if let Some((key, value)) = comment.split_once(": ") {
            match key {
                "AKAZA_DATA_VERSION" | "BUILD_TIMESTAMP" => {
                    println!("{key}: {value}");
                    found = true;
                }
                _ => {}
            }
        }
    }

    if !found {
        println!("AKAZA_DATA_VERSION: (not set)");
        println!("BUILD_TIMESTAMP: (not set)");
    }

    Ok(())
}
