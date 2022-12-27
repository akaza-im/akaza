use std::io::Error;

use crate::trie::{Trie, TrieBuilder};

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
