use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Local;
use log::info;
use redb::{Database, ReadableTable, TableDefinition};
use rustc_hash::FxHashMap;

use libakaza::lm::base::SystemUnigramLM;

use crate::utils::{get_file_list, normalize_num_token, parse_dir_weight};
use crate::wordcnt::wordcnt_skip_bigram::WordcntSkipBigramBuilder;
use crate::wordcnt::wordcnt_unigram::WordcntUnigram;

/// redb テーブル: キーは (i32, i32) を 8 バイトにエンコード、値は f64（重み付き集計用）
const SKIP_BIGRAM_TABLE: TableDefinition<&[u8], f64> = TableDefinition::new("skip_bigram");

fn encode_key(id1: i32, id2: i32) -> [u8; 8] {
    let mut buf = [0u8; 8];
    buf[..4].copy_from_slice(&id1.to_be_bytes());
    buf[4..].copy_from_slice(&id2.to_be_bytes());
    buf
}

fn decode_key(buf: &[u8]) -> (i32, i32) {
    let id1 = i32::from_be_bytes(buf[..4].try_into().unwrap());
    let id2 = i32::from_be_bytes(buf[4..].try_into().unwrap());
    (id1, id2)
}

/// skip-bigram (w_{i-2}, w_i) をコーパスからカウントして TRIE ファイルを生成する。
pub fn make_stats_system_skip_bigram_lm(
    threshold: u32,
    corpus_dirs: &Vec<String>,
    unigram_trie_file: &str,
    skip_bigram_trie_file: &str,
) -> Result<()> {
    let unigram_lm = WordcntUnigram::load(unigram_trie_file)?;
    info!(
        "Unigram system lm: {} threshold={}",
        unigram_lm.num_keys(),
        threshold
    );

    let unigram_map = unigram_lm
        .as_hash_map()
        .iter()
        .map(|(key, (word_id, _))| (key.clone(), *word_id))
        .collect::<FxHashMap<_, _>>();
    let reverse_unigram_map = unigram_map
        .iter()
        .map(|(key, word_id)| (*word_id, key.to_string()))
        .collect::<FxHashMap<_, _>>();

    let mut file_list: Vec<(PathBuf, f64)> = Vec::new();
    for corpus_dir in corpus_dirs {
        let (dir, weight) = parse_dir_weight(corpus_dir);
        info!("Corpus dir: {} (weight={})", dir, weight);
        let list = get_file_list(Path::new(&dir))?;
        for x in list {
            file_list.push((x, weight));
        }
    }

    // redb でオンディスク集計
    let tmp_db = tempfile::NamedTempFile::new()?;
    let db = Database::create(tmp_db.path())?;

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

        let mut batch_stats: FxHashMap<(i32, i32), f64> = FxHashMap::default();
        for (path_buf, weight) in chunk {
            info!(
                "  Counting {} (weight={})",
                path_buf.to_string_lossy(),
                weight
            );
            let file = File::open(path_buf)?;

            for line in BufReader::new(file).lines() {
                let line = line?;
                let line = line.trim();

                // 行内の全単語の word_id を収集
                let word_ids: Vec<Option<i32>> = line
                    .split(' ')
                    .filter(|w| !w.is_empty())
                    .map(|word| {
                        let normalized = normalize_num_token(word);
                        unigram_map.get(normalized.as_ref()).copied()
                    })
                    .collect();

                // skip-bigram: (w[i-2], w[i]) のペアをカウント
                for i in 2..word_ids.len() {
                    if let (Some(id1), Some(id2)) = (word_ids[i - 2], word_ids[i]) {
                        *batch_stats.entry((id1, id2)).or_insert(0.0) += weight;
                    }
                }
            }
        }

        // バッチ分をまとめて 1 トランザクションで DB にマージ
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(SKIP_BIGRAM_TABLE)?;
            for ((id1, id2), cnt) in &batch_stats {
                let key = encode_key(*id1, *id2);
                let prev = table.get(key.as_slice())?.map(|v| v.value()).unwrap_or(0.0);
                table.insert(key.as_slice(), prev + cnt)?;
            }
        }
        write_txn.commit()?;
    }

    // dump skip-bigram text file
    let dumpfname = format!(
        "work/dump/skip-bigram-{}.txt",
        Local::now().format("%Y%m%d-%H%M%S")
    );
    println!("Dump to text file: {dumpfname}");
    let mut dump_file = File::create(&dumpfname)?;

    info!("Generating trie file");
    let mut builder = WordcntSkipBigramBuilder::default();

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(SKIP_BIGRAM_TABLE)?;
    for entry in table.iter()? {
        let entry = entry?;
        let (word_id1, word_id2) = decode_key(entry.0.value());
        let cnt_f64 = entry.1.value();
        let cnt = cnt_f64.round() as u32;

        // dump (cnt > 16)
        if cnt > 16 {
            if let (Some(word1), Some(word2)) = (
                reverse_unigram_map.get(&word_id1),
                reverse_unigram_map.get(&word_id2),
            ) {
                dump_file.write_fmt(format_args!("{cnt}\t{word1}\t{word2}\n"))?;
            }
        }

        // threshold で足切り
        if cnt > threshold {
            builder.add(word_id1, word_id2, cnt);
        }
    }

    info!("Writing {}", skip_bigram_trie_file);
    builder.save(skip_bigram_trie_file)?;

    println!("DONE");
    Ok(())
}
