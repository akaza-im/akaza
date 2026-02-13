use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use crate::utils::normalize_num_token;
use crate::wordcnt::wordcnt_unigram::WordcntUnigramBuilder;

/// 統計的かな漢字変換のためのユニグラムシステム言語モデルの作成
///
/// wfreq ファイルを開いてパースし、ユニグラム言語モデルファイルを作成して保存する。
pub fn make_stats_system_unigram_lm(srcpath: &str, dstpath: &str) -> anyhow::Result<()> {
    // 16 はヒューリスティックな値。調整の余地。
    let threshold = 16_u32;

    let mut wordcnt = parse_wfreq(srcpath, threshold)?;
    wordcnt.insert("__BOS__/__BOS__".to_string(), 0);
    wordcnt.insert("__EOS__/__EOS__".to_string(), 0);
    if wordcnt.len() >= 8388608 {
        // edge cost 言語モデルファイルの容量を小さく保つために
        // 3 byte に ID が収めるようにする。
        // よって、最大でも 8,388,608 単語までになるように vocab を制限する。
        // 現実的な線で切っても、500万単語ぐらいで十分。
        panic!("too much words in wfreq file: {srcpath}");
    }

    let mut builder = WordcntUnigramBuilder::default();
    for (word, score) in &wordcnt {
        builder.add(word.as_str(), *score);
    }

    println!("Writing {dstpath}");
    builder.save(dstpath)?;

    Ok(())
}

fn parse_wfreq(src_file: &str, threshold: u32) -> anyhow::Result<HashMap<String, u32>> {
    let file = File::open(src_file)?;
    let mut map: HashMap<String, u32> = HashMap::new();

    for line in BufReader::new(file).lines() {
        let line = line?;
        let trimmed = line.trim();
        let Some((word, cnt_str)) = trimmed.split_once('\t') else {
            log::warn!("Skipping malformed wfreq line: {:?}", trimmed);
            continue;
        };
        let cnt: u32 = match cnt_str.parse() {
            Ok(v) => v,
            Err(_) => {
                log::warn!("Skipping unparseable count in wfreq line: {:?}", trimmed);
                continue;
            }
        };
        // 数字トークンを <NUM> に正規化してカウントを集約
        let normalized = normalize_num_token(word);
        *map.entry(normalized.into_owned()).or_insert(0) += cnt;
    }
    // threshold フィルタは集約後に適用（個別では threshold 以下でも集約後に超える場合がある）
    map.retain(|_, cnt| *cnt > threshold);
    Ok(map)
}
