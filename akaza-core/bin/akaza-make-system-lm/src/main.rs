use std::env;
use std::fs::File;
use std::io::{BufReader, prelude::*};

use libakaza::lm::{SystemBigramLMBuilder, SystemUnigramLMBuilder, SystemUnigramLM};

// e.g.g 倉庫会社/そうこがいしゃ -6.973789593503506
unsafe fn process_unigram(srcpath: &String, dstpath: &String) {
    let file = File::open(srcpath).expect("Open {txtfile} correctly.");

    let builder = SystemUnigramLMBuilder::new();
    let mut i: u64 = 0;
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let v: Vec<&str> = line.trim().split(" ").collect();
        let word = v[0];
        let score = v[1];
        let score: f32 = score.parse().unwrap();

        builder.add(&word.to_string(), score);

        i += 1;
        if i >= 8388608 {
            // 3 byte に ID が収まる必要がある
            panic!("too much words.");
        }
    }

    println!("Writing {}", dstpath);
    builder.save(dstpath)
        .unwrap();
}


unsafe fn process_2gram(unigram: &SystemUnigramLM, srcpath: &String, dstpath: &String) {
    let file = File::open(srcpath).unwrap();

    let builder = SystemBigramLMBuilder::new();

    for line in BufReader::new(file).lines() {
        fn parse_2gram_line(line: &String) -> (String, String, f32) {
            let tokens: Vec<&str> = line.split(" ").collect();
            if tokens.len() != 2 {
                println!("Invalid tokens: {:?}", tokens);
                panic!()
            }
            let words: &str = tokens[0];
            let score = tokens[1];

            let words: Vec<&str> = words.split("\t").collect();

            let word1 = words[0];
            let word2 = words[1];
            let score = score.parse().unwrap();
            return (word1.to_string(), word2.to_string(), score);
        }

        let line = line.unwrap();
        let (word1, word2, score) = parse_2gram_line(&line);

        // println!("word1='{}' word2='{}' score='{}'", word1, word2, score);

        let word_id1 = unigram.find_unigram(&word1.to_string());
        let word_id2 = unigram.find_unigram(&word2.to_string());
        if word_id1.is_none() || word_id2.is_none() {
            println!("Unknown word(not in unigram dict): word1='{}' word2='{}'", word1, word2);
            continue;
        }

        builder.add(word_id1.unwrap(), word_id2.unwrap(), score);
    }

    builder.save(dstpath).unwrap();
}


fn main() {
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

    unsafe { process_unigram(unigram_src, unigram_dst); }


    // 2gram ファイルから読む
    // 2gram ファイルを書いていく。
    println!("Bigram {} to {}", bigram_src, bigram_dst);

    let unigram_lm = unsafe { SystemUnigramLM::load(unigram_dst).unwrap() };
    unsafe { println!("Unigram system lm: {}", unigram_lm.num_keys()); }
    unsafe { process_2gram(&unigram_lm, bigram_src, bigram_dst); }

    println!("DONE");
}
