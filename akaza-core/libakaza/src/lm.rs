use std::io::Error;

use crate::trie::{Trie, TrieBuilder};

// このあたりは C++ 時代の Spec。
// rust 版ではいったん、簡単のために id を sizeof(usize) バイト書いている。

/*

-- 1gram

    {word} # in utf-8
    0xff   # marker
    packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608(2**24/2)
    packed float  # score: 4 bytes

-- 2gram

    {word1 ID}    # 3 bytes
    {word2 ID}    # 3 bytes
    packed float  # score: 4 bytes

 */

// TODO: move to libakazac
pub struct SystemUnigramLMBuilder {
    builder: TrieBuilder,
}

impl SystemUnigramLMBuilder {
    pub unsafe fn new() -> SystemUnigramLMBuilder {
        SystemUnigramLMBuilder { builder: TrieBuilder::new() }
    }

    pub unsafe fn add(&self, word: &String, score: f32) {
        let key = [
            word.as_bytes(), b"\xff", score.to_le_bytes().as_slice()
        ].concat();
        self.builder.add(key);
    }

    pub unsafe fn save(&self, fname: &String) -> std::io::Result<()> {
        return self.builder.save(fname);
    }
}

pub struct SystemUnigramLM {
    trie: Trie,
}

impl SystemUnigramLM {
    pub unsafe fn num_keys(&self) -> usize {
        return self.trie.num_keys();
    }

    pub unsafe fn load(fname: &String) -> Result<SystemUnigramLM, Error> {
        println!("Reading {}", fname);
        return match Trie::load(fname) {
            Ok(trie) => Ok(SystemUnigramLM { trie }),
            Err(err) => Err(err)
        };
    }

    pub unsafe fn find_unigram(&self, word: &String) -> Option<usize> {
        let query = [word.as_bytes(), b"\xff"].concat();
        let got = self.trie.predictive_search(query);
        return if got.is_empty() {
            None
        } else {
            Some(got[0].id)
        };
    }
}

pub struct SystemBigramLMBuilder {
    builder: TrieBuilder,
}

impl SystemBigramLMBuilder {
    pub unsafe fn new() -> SystemBigramLMBuilder {
        SystemBigramLMBuilder { builder: TrieBuilder::new() }
    }

    pub unsafe fn add(&self, word_id1: usize, word_id2: usize, score: f32) {
        let mut key: Vec<u8> = Vec::new();
        key.extend(word_id1.to_le_bytes());
        key.extend(word_id2.to_le_bytes());
        key.extend(score.to_le_bytes());
        self.builder.add(key);
    }

    pub unsafe fn save(&self, ofname: &String) -> std::io::Result<()> {
        return self.builder.save(ofname);
    }
}
