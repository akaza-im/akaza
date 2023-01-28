use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use log::{info, warn};

/// wfreq (単語の発生頻度表)から vocab (語彙ファイル)を作成する。
pub fn vocab(src_file: &str, dst_file: &str, threshold: u32) -> anyhow::Result<()> {
    info!(
        "vocab: {} => {}, threshold={}",
        src_file, dst_file, threshold
    );

    let ifp = File::open(src_file)?;
    let mut ofp = File::create(dst_file.to_string() + ".tmp")?;
    for line in BufReader::new(ifp).lines() {
        let line = line?;
        let line = line.trim();
        let (word, cnt) = line.split_once('\t').unwrap();
        if word.starts_with(' ') || word.starts_with('/') {
            warn!("Invalid word: {:?}", line);
            continue;
        }
        if !word.contains('/') {
            warn!("Invalid word: {:?}", line);
            continue;
        }
        let cnt: u32 = cnt.parse()?;
        if cnt > threshold {
            ofp.write_fmt(format_args!("{word}\n"))?;
        }
    }
    fs::rename(dst_file.to_owned() + ".tmp", dst_file)?;

    Ok(())
}
