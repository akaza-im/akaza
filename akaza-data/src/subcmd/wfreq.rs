use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use log::{info, warn};
use redb::{Database, ReadableTable, TableDefinition};
use regex::Regex;

use crate::utils::{get_file_list, parse_dir_weight};

const WFREQ_TABLE: TableDefinition<&str, f64> = TableDefinition::new("wfreq");

/// 単語の発生確率を数え上げる。
/// redb をオンディスク KV として使用し、大規模コーパスでも OOM しない。
/// src_dirs は `"path:weight"` 形式に対応する（weight 省略時は 1.0）。
pub fn wfreq(src_dirs: &Vec<String>, dst_file: &str) -> anyhow::Result<()> {
    info!("wfreq: {:?} => {}", src_dirs, dst_file);

    let mut file_list: Vec<(PathBuf, f64)> = Vec::new();
    for src_dir in src_dirs {
        let (dir, weight) = parse_dir_weight(src_dir);
        info!("Source: {} (weight={})", dir, weight);
        let list = get_file_list(Path::new(&dir))?;
        for x in list {
            file_list.push((x, weight));
        }
    }

    // 一時ファイルに redb データベースを作成
    let tmp_db = tempfile::NamedTempFile::new()?;
    let db = Database::create(tmp_db.path())?;

    // 複数ファイルをまとめて 1 トランザクションで commit することで、
    // トランザクションオーバーヘッドを削減する。
    const BATCH_SIZE: usize = 100;
    for (batch_idx, chunk) in file_list.chunks(BATCH_SIZE).enumerate() {
        let batch_start = batch_idx * BATCH_SIZE + 1;
        let batch_end = (batch_start + chunk.len() - 1).min(file_list.len());
        info!(
            "Processing batch {}-{}/{} ({} files)",
            batch_start,
            batch_end,
            file_list.len(),
            chunk.len()
        );

        // バッチ内の全ファイルをメモリ上で集計
        let mut batch_stats: rustc_hash::FxHashMap<String, f64> = rustc_hash::FxHashMap::default();
        for (path_buf, weight) in chunk {
            info!("  Processing {}", path_buf.to_string_lossy());
            let file = File::open(path_buf)?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                let line = line.trim();
                for word in line.split(' ') {
                    if word.is_empty() || word.as_bytes()[0] == b'/' || word.as_bytes()[0] == b' ' {
                        continue;
                    }
                    if word.contains('\u{200f}') {
                        warn!("The document contains RTL character");
                        continue;
                    }
                    // ASCII 制御文字を含むワードはゴミデータなのでスキップ
                    if word.bytes().any(|b| b.is_ascii_control()) {
                        continue;
                    }
                    *batch_stats.entry(word.to_string()).or_insert(0.0) += weight;
                }
            }
        }

        // バッチ分をまとめて 1 トランザクションで DB にマージ
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(WFREQ_TABLE)?;
            for (word, cnt) in &batch_stats {
                let prev = table.get(word.as_str())?.map(|v| v.value()).unwrap_or(0.0);
                table.insert(word.as_str(), prev + cnt)?;
            }
        }
        write_txn.commit()?;
    }

    // 結果をファイルに書いていく
    info!("Write to {}", dst_file);
    // 明らかに不要なワードが登録されているのを除外する。
    // カタカナ二文字系は全般的にノイズになりがちだが、Wikipedia/青空文庫においては
    // 架空の人物や実在の人物の名前として使われがちなので、消す。
    let re = Regex::new("^[\u{30A0}-\u{30FF}]{2}/[\u{3040}-\u{309F}]{2}$")?;
    let mut ofp = File::create(dst_file.to_string() + ".tmp")?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(WFREQ_TABLE)?;
    // redb の BTree は key 順にイテレートされるのでソート不要
    for entry in table.iter()? {
        let entry = entry?;
        let word = entry.0.value();
        let cnt_f64 = entry.1.value();
        let cnt = cnt_f64.round() as u32;
        if cnt == 0 {
            continue;
        }
        if re.is_match(word) {
            info!("Skip 2 character katakana entry: {}", word);
            continue;
        }
        ofp.write_fmt(format_args!("{word}\t{cnt}\n"))?;
    }

    fs::rename(dst_file.to_owned() + ".tmp", dst_file)?;

    // tmp_db は drop 時に自動削除される
    Ok(())
}
