use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use log::{info, warn, trace};
use rayon::prelude::*;

use crate::utils::get_file_list;

/// 単語の発生確率を数え上げる。
pub fn wfreq(src_dir: &str, dst_file: &str) -> anyhow::Result<()> {
    info!("wfreq: {} => {}", src_dir, dst_file);

    let file_list = get_file_list(Path::new(src_dir))?;
    let results = file_list
        .par_iter()
        .map(|path_buf| -> anyhow::Result<HashMap<String, u32>> {
            // ファイルを読み込んで、HashSet に単語数を数え上げる。
            info!("Processing {}", path_buf.to_string_lossy());
            let file = File::open(path_buf)?;
            let mut stats: HashMap<String, u32> = HashMap::new();
            for line in BufReader::new(file).lines() {
                let line = line?;
                let line = line.trim();
                let words = line.split(' ').collect::<Vec<_>>();
                for word in words {
                    if word.is_empty() {
                        continue;
                    }
                    if word.contains('\u{200f}') {
                        warn!("The document contains RTL character");
                        continue;
                    }
                    if word.starts_with('/') {
                        trace!("Invalid word: {}", word);
                        continue;
                    }
                    if word.starts_with(' ') {
                        trace!("Invalid word: {}", word);
                        continue;
                    }
                    *stats.entry(word.to_string()).or_insert(0) += 1;
                }
            }
            Ok(stats)
        })
        .collect::<Vec<_>>();

    // 最終結果ファイルは順番が安定な方がよいので BTreeMap を採用。
    info!("Merging");
    let mut retval: BTreeMap<String, u32> = BTreeMap::new();
    for result in results {
        // このへんでマージを行う。
        let result = result?;
        for (word, cnt) in result {
            *retval.entry(word.to_string()).or_insert(0) += cnt;
        }
    }

    // 結果をファイルに書いていく
    info!("Write to {}", dst_file);
    let mut ofp = File::create(dst_file.to_string() + ".tmp")?;
    for (word, cnt) in retval {
        ofp.write_fmt(format_args!("{}\t{}\n", word, cnt))?;
    }
    fs::rename(dst_file.to_owned() + ".tmp", dst_file)?;

    Ok(())
}
