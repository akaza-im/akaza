use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Result;
use anyhow::{anyhow, Context};
use log::info;
use rayon::prelude::*;

use libakaza::lm::system_bigram::{SystemBigramLM, SystemBigramLMBuilder};
use libakaza::lm::system_unigram_lm::SystemUnigramLM;

use crate::subcmd::make_stats_system_unigram_lm::calc_score;
use crate::utils::get_file_list;

pub fn make_stats_system_bigram_lm(
    threshold: u32,
    corpus_dir: &String,
    unigram_trie_file: &str,
    bigram_trie_file: &str,
) -> Result<()> {
    // まずは unigram の language model を読み込む
    let unigram_lm = SystemUnigramLM::load(unigram_trie_file)?;
    info!(
        "Unigram system lm: {} threshold={}",
        unigram_lm.num_keys(),
        threshold
    );

    let unigram_map = unigram_lm.as_id_map();

    // 次に、コーパスをスキャンして bigram を読み取る。
    let file_list = get_file_list(Path::new(corpus_dir))?;
    let results = file_list
        .par_iter()
        .map(|src| count_bigram(src, &unigram_map))
        .collect::<Vec<_>>();

    // 集計した結果をマージする
    let mut merged: HashMap<(i32, i32), u32> = HashMap::new();
    for result in results {
        let result = result?;
        for (word_ids, cnt) in result {
            *merged.entry(word_ids).or_insert(0) += cnt;
        }
    }

    // スコアを計算する
    let scoremap = make_score_map(threshold, merged);

    // 結果を書き込む
    let mut builder = SystemBigramLMBuilder::default();
    for ((word_id1, word_id2), score) in scoremap {
        builder.add(word_id1, word_id2, score);
    }
    builder.save(bigram_trie_file)?;

    validation(unigram_trie_file, bigram_trie_file)?;

    println!("DONE");
    Ok(())
}

// unigram のロジックと一緒なのでまとめたい。
fn make_score_map(threshold: u32, wordcnt: HashMap<(i32, i32), u32>) -> HashMap<(i32, i32), f32> {
    // 総出現単語数
    let c = wordcnt.values().sum();
    // 単語の種類数
    let v = wordcnt.keys().count();
    wordcnt
        .iter()
        .filter(|(_word, cnt)| *cnt > &threshold)
        .map(|(word, cnt)| (*word, calc_score(*cnt, c, v)))
        .collect::<HashMap<_, _>>()
}

fn count_bigram(
    src: &PathBuf,
    unigram_lm: &HashMap<String, i32>,
) -> anyhow::Result<HashMap<(i32, i32), u32>> {
    info!("Counting {}", src.to_string_lossy());
    let file = File::open(src)?;
    let mut map: HashMap<(i32, i32), u32> = HashMap::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        let line = line.trim();
        let words = line.split(' ').collect::<Vec<_>>();
        if words.len() < 2 {
            continue;
        }
        let word_ids = words
            .iter()
            .map(|word| unigram_lm.get(&word.to_string()))
            .collect::<Vec<_>>();

        for i in 0..(word_ids.len() - 1) {
            let Some(maybe_word_id1) = word_ids[i] else {
                continue;
            };
            let Some(maybe_word_id2) = word_ids[i + 1] else {
                continue;
            };
            *map.entry((*maybe_word_id1, *maybe_word_id2)).or_insert(0) += 1;
        }
    }
    Ok(map)
}

// 言語モデルファイルが正確に生成されたか確認を実施する
fn validation(unigram_dst: &str, bigram_dst: &str) -> Result<()> {
    let unigram = SystemUnigramLM::load(unigram_dst).unwrap();
    let bigram = SystemBigramLM::load(bigram_dst).unwrap();

    let word1 = "私/わたし";

    let (word1_id, watshi_cost) = unigram
        .find(word1)
        .ok_or_else(|| anyhow!("Cannot find '{}' in unigram dict.", word1))?;
    println!("word1_id={} word1_cost={}", word1_id, watshi_cost);

    let word2 = "から/から";
    let (word2_id, word2_cost) = unigram
        .find(word2)
        .ok_or_else(|| anyhow!("Cannot find '{}' in unigram dict.", word1))?;
    println!("word2_id={} word2_cost={}", word2_id, word2_cost);

    bigram.get_edge_cost(word1_id, word2_id).with_context(|| {
        format!(
            "Get bigram entry: '{} -> {}' {},{}",
            word1, word2, word1_id, word2_id
        )
    })?;

    Ok(())
}
