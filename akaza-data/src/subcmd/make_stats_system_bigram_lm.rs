use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use log::info;
use redb::{Database, ReadableTable, TableDefinition};
use rustc_hash::FxHashMap;

use libakaza::lm::base::{SystemBigramLM, SystemUnigramLM};

use crate::utils::{get_file_list, normalize_num_token, parse_dir_weight};
use crate::wordcnt::wordcnt_bigram::{WordcntBigram, WordcntBigramBuilder};
use crate::wordcnt::wordcnt_unigram::WordcntUnigram;

/// redb テーブル: キーは (i32, i32) を 8 バイトにエンコード、値は f64（重み付き集計用）
const BIGRAM_TABLE: TableDefinition<&[u8], f64> = TableDefinition::new("bigram");

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

pub fn make_stats_system_bigram_lm(
    threshold: u32,
    corpus_dirs: &Vec<String>,
    unigram_trie_file: &str,
    bigram_trie_file: &str,
) -> Result<()> {
    // まずは unigram の language model を読み込む
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

    // 次に、コーパスをスキャンして bigram を読み取る。
    // corpus_dirs は "path:weight" 形式に対応する（weight 省略時は 1.0）。
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

    let bos_id = unigram_map.get("__BOS__/__BOS__").copied();
    let eos_id = unigram_map.get("__EOS__/__EOS__").copied();

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

                let mut prev_word_id: Option<i32> = bos_id;
                let mut last_word_id: Option<i32> = None;
                let mut has_words = false;

                for word in line.split(' ') {
                    if word.is_empty() {
                        continue;
                    }
                    has_words = true;
                    let normalized = normalize_num_token(word);
                    let cur_word_id = unigram_map.get(normalized.as_ref()).copied();

                    if let (Some(prev), Some(cur)) = (prev_word_id, cur_word_id) {
                        *batch_stats.entry((prev, cur)).or_insert(0.0) += weight;
                    }

                    prev_word_id = cur_word_id;
                    last_word_id = cur_word_id;
                }

                if !has_words {
                    continue;
                }

                // 最後の単語 → EOS
                if let (Some(last), Some(eos)) = (last_word_id, eos_id) {
                    *batch_stats.entry((last, eos)).or_insert(0.0) += weight;
                }
            }
        }

        // バッチ分をまとめて 1 トランザクションで DB にマージ
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(BIGRAM_TABLE)?;
            for ((id1, id2), cnt) in &batch_stats {
                let key = encode_key(*id1, *id2);
                let prev = table.get(key.as_slice())?.map(|v| v.value()).unwrap_or(0.0);
                table.insert(key.as_slice(), prev + cnt)?;
            }
        }
        write_txn.commit()?;
    }

    // dump bigram text file.
    let dumpfname = format!(
        "work/dump/bigram-{}.txt",
        Local::now().format("%Y%m%d-%H%M%S")
    );
    println!("Dump to text file: {dumpfname}");
    let mut dump_file = File::create(&dumpfname)?;

    // 結果を書き込む
    info!("Generating trie file");
    let mut builder = WordcntBigramBuilder::default();

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(BIGRAM_TABLE)?;
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

    info!("Writing {}", bigram_trie_file);
    builder.save(bigram_trie_file)?;

    validation(unigram_trie_file, bigram_trie_file)?;

    println!("DONE");
    Ok(())
}

// 言語モデルファイルが正確に生成されたか確認を実施する
fn validation(unigram_dst: &str, bigram_dst: &str) -> Result<()> {
    let unigram = WordcntUnigram::load(unigram_dst).unwrap();
    let bigram = WordcntBigram::load(bigram_dst).unwrap();

    let word1 = "私/わたし";

    let (word1_id, watshi_cost) = unigram
        .find(word1)
        .ok_or_else(|| anyhow!("Cannot find '{}' in unigram dict.", word1))?;
    println!("word1_id={word1_id} word1_cost={watshi_cost}");

    let word2 = "から/から";
    let (word2_id, word2_cost) = unigram
        .find(word2)
        .ok_or_else(|| anyhow!("Cannot find '{}' in unigram dict.", word1))?;
    println!("word2_id={word2_id} word2_cost={word2_cost}");

    bigram
        .get_edge_cost(word1_id, word2_id)
        .with_context(|| format!("Get bigram entry: '{word1} -> {word2}' {word1_id},{word2_id}"))?;

    Ok(())
}
