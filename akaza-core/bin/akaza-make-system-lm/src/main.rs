use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use anyhow::Result;
use anyhow::{anyhow, Context};

use libakaza::lm::system_bigram::{SystemBigramLM, SystemBigramLMBuilder};
use libakaza::lm::system_unigram_lm::{SystemUnigramLM, SystemUnigramLMBuilder};

// e.g.g 倉庫会社/そうこがいしゃ -6.973789593503506
fn process_unigram(srcpath: &String, dstpath: &String) {
    let file = File::open(srcpath).expect("Open {txtfile} correctly.");

    let mut builder = SystemUnigramLMBuilder::default();
    let mut i: u64 = 0;
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let (word, score) = line.trim().split_once(' ').unwrap();
        let score: f32 = score.parse().unwrap();

        builder.add(word, score);

        i += 1;
        if i >= 8388608 {
            // edge cost 言語モデルファイルの容量を小さく保つために
            // 3 byte に ID が収めるようにする。
            // よって、最大でも 8,388,608 単語までになるように vocab を制限する。
            // 現実的な線で切っても、500万単語ぐらいで十分。
            panic!("too much words.");
        }
    }

    println!("Writing {}", dstpath);
    builder.save(dstpath).unwrap();
}

fn process_2gram(unigram: &SystemUnigramLM, srcpath: &str, dstpath: &str) -> Result<()> {
    let file = File::open(srcpath)?;

    let mut builder = SystemBigramLMBuilder::default();

    for line in BufReader::new(file).lines() {
        fn parse_2gram_line(line: &str) -> Result<(String, String, f32)> {
            let tokens: Vec<&str> = line.split(' ').collect();
            if tokens.len() != 2 {
                println!("Invalid tokens: {:?}", tokens);
                panic!()
            }
            let words: &str = tokens[0];
            let score = tokens[1];

            let Some((word1, word2)) = words.split_once('\t') else {
                return Err(anyhow!("Cannot split words: {}", words));
            };
            let score = score.parse().unwrap();
            Ok((word1.to_string(), word2.to_string(), score))
        }

        let line = line.unwrap();
        let (word1, word2, score) = parse_2gram_line(&line)?;

        // println!("word1='{}' word2='{}' score='{}'", word1, word2, score);

        let Some((word_id1, _)) = unigram.find(&word1.to_string()) else {
            println!("Can't find '{}' in unigram data", word2);
            continue;
        };
        let Some((word_id2, _)) = unigram
            .find(&word2.to_string()) else {
            println!("Can't find '{}' in unigram data", word2);
            continue;
        };
        /*
        {
            // debugging
            if word1 == "私/わたし" {
                println!("Inserting: {}({}) {}({})", word1, word_id1, word2, word_id2)
            }
        }
         */

        builder.add(word_id1, word_id2, score);
    }

    builder.save(dstpath).unwrap();
    Ok(())
}

fn main() -> Result<()> {
    // 1gram ファイルから読む。
    // 1gram の map<string, int> の ID mapping を作成する
    // 1gram データを書いていく。

    // "work/jawiki.merged-1gram.txt" "akaza_data/data/lm_v2_1gram.trie"
    // "work/jawiki.merged-2gram.txt" "akaza_data/data/lm_v2_2gram.trie"

    let args: Vec<String> = env::args().collect();
    let unigram_src = &args[1];
    let unigram_dst = &args[2];
    let bigram_src = &args[3];
    let bigram_dst = &args[4];

    // std::map<std::string, uint32_t> word2id;
    // "work/jawiki.merged-1gram.txt" -> "akaza_data/data/lm_v2_1gram.trie"
    println!("Unigram {} to {}", unigram_src, unigram_dst);

    process_unigram(unigram_src, unigram_dst);

    // 2gram ファイルから読む
    // 2gram ファイルを書いていく。
    println!("Bigram {} to {}", bigram_src, bigram_dst);

    let unigram_lm = SystemUnigramLM::load(unigram_dst).unwrap();
    println!("Unigram system lm: {}", unigram_lm.num_keys());
    process_2gram(&unigram_lm, bigram_src, bigram_dst)?;

    validation(unigram_dst, bigram_dst)?;

    println!("DONE");
    Ok(())
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
