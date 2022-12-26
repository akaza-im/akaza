use std::env;
use std::fs::File;
use std::io::{BufReader, Error, prelude::*};

use libakaza::trie::{Trie, TrieBuilder};

/**

# 1gram

    {word} # in utf-8
    0xff   # marker
    packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608(2**24/2)
    packed float  # score: 4 bytes

# 2gram

    {word1 ID}    # 3 bytes
    {word2 ID}    # 3 bytes
    packed float  # score: 4 bytes

 */

// TODO: move to libakaza?
struct SystemUnigramLMBuilder {
    builder: TrieBuilder,
}

impl SystemUnigramLMBuilder {
    pub unsafe fn new() -> SystemUnigramLMBuilder {
        SystemUnigramLMBuilder { builder: TrieBuilder::new() }
    }

    pub unsafe fn add(&self, word: &String, score: f32) {
        let key = [
            word.as_bytes(), b"\xff", score.to_le_bytes().as_slice(), b"\x00"
        ].concat();
        self.builder.add(key);
    }

    pub unsafe fn save(&self, fname: &String) -> std::io::Result<()> {
        return self.builder.save(fname);
    }
}

struct SystemUnigramLM {
    trie: Trie,
}

impl SystemUnigramLM {
    unsafe fn load(fname: &String) -> Result<SystemUnigramLM, Error> {
        return match Trie::load(fname) {
            Ok(trie) => Ok(SystemUnigramLM { trie }),
            Err(err) => Err(err)
        };
    }

    unsafe fn find_unigram(&self, word: &String) -> Option<i32> {
        let query = [word.as_bytes(), b"\xff\x00"].concat();
        let got = self.trie.predictive_search(query);
        return if got.is_empty() {
            None
        } else {
            Some(got[0].id)
        };
    }
}

struct SystemBigramLMBuilder {
    builder: TrieBuilder,
}

impl SystemBigramLMBuilder {
    unsafe fn new() -> SystemBigramLMBuilder {
        SystemBigramLMBuilder { builder: TrieBuilder::new() }
    }

    unsafe fn add(&self, word_id1: i32, word_id2: i32, score: f32) {
        let key = [word_id1.to_le_bytes(), word_id2.to_le_bytes(), score.to_le_bytes()].concat();
        self.builder.add(key);
    }

    unsafe fn save(&self, ofname: &String) -> std::io::Result<()> {
        return self.builder.save(ofname);
    }
}

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

    builder.save(dstpath)
        .unwrap();
}

unsafe fn process_2gram(unigram: &SystemUnigramLM, srcpath: &String, dstpath: &String) {
    let file = File::open(srcpath).unwrap();

    let builder = SystemBigramLMBuilder::new();

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.split(" ").collect();
        let word1 = tokens[0];
        let word2 = tokens[1];
        let score = tokens[2].parse().unwrap();

        println!("word1='{}' word2='{}' score='{}'", word1, word2, score);

        let word_id1 = unigram.find_unigram(&word1.to_string()).unwrap();
        let word_id2 = unigram.find_unigram(&word2.to_string()).unwrap();

        builder.add(word_id1, word_id2, score);
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
    unsafe { process_2gram(&unigram_lm, bigram_src, bigram_dst); }

    println!("DONE");
}
