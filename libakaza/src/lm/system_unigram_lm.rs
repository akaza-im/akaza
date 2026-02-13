use std::collections::HashMap;

use anyhow::{bail, Result};
use log::{info, warn};

use rsmarisa::{Agent, Keyset, Trie};

use crate::cost::calc_cost;
use crate::lm::base::SystemUnigramLM;

/*
   {word} # in utf-8
   0xff   # marker
   packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608(2**24/2)
   packed float  # score: 4 bytes
*/

const UNIQUE_WORDS_KEY: &str = "__UNIQUE_WORDS__";
const TOTAL_WORDS_KEY: &str = "__TOTAL_WORDS__";

/**
 * unigram 言語モデル。
 * 「漢字/かな」に対して、発生確率スコアを保持している。
 */
#[derive(Default)]
pub struct MarisaSystemUnigramLMBuilder {
    data: Vec<(String, f32)>,
}

impl MarisaSystemUnigramLMBuilder {
    pub fn add(&mut self, word: &str, score: f32) {
        self.data.push((word.to_string(), score));
    }

    pub fn keyset(&mut self) -> Result<Keyset> {
        let mut keyset = Keyset::new();
        for (kanji, score) in &self.data {
            // 区切り文字をいれなくても、末尾の4バイトを取り出せば十分な気がしないでもない。。
            // 先頭一致にして、+4バイトになるものを探せばいいはず。
            // 最適化の余地だけど、現実的には空間効率よりも速度のほうが重要かもしれない。
            let key = [
                kanji.as_bytes(),
                b"\xff",
                score.to_le_bytes().as_slice(), // バイナリにしてデータ容量を節約する
            ]
            .concat();
            keyset.push_back_bytes(&key, 1.0)?;
        }
        Ok(keyset)
    }

    pub fn set_total_words(&mut self, total_words: u32) -> &mut Self {
        self.add(TOTAL_WORDS_KEY, total_words as f32);
        self
    }

    pub fn set_unique_words(&mut self, unique_words: u32) -> &mut Self {
        self.add(UNIQUE_WORDS_KEY, unique_words as f32);
        self
    }

    pub fn save(&mut self, fname: &str) -> Result<()> {
        let mut keyset = self.keyset()?;
        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);
        trie.save(fname)?;
        Ok(())
    }

    pub fn build(&mut self) -> Result<MarisaSystemUnigramLM> {
        let mut keyset = self.keyset()?;
        let mut trie = Trie::new();
        trie.build(&mut keyset, 0);
        let Some((_, total_words)) = MarisaSystemUnigramLM::find_from_trie(&trie, TOTAL_WORDS_KEY)
        else {
            bail!("Missing key for {}", TOTAL_WORDS_KEY);
        };
        let Some((_, unique_words)) =
            MarisaSystemUnigramLM::find_from_trie(&trie, UNIQUE_WORDS_KEY)
        else {
            bail!("Missing key for {}", UNIQUE_WORDS_KEY);
        };
        Ok(MarisaSystemUnigramLM {
            trie,
            total_words: total_words as u32,
            unique_words: unique_words as u32,
        })
    }
}

pub struct MarisaSystemUnigramLM {
    trie: Trie,
    total_words: u32,
    unique_words: u32,
}

impl MarisaSystemUnigramLM {
    pub fn num_keys(&self) -> usize {
        self.trie.num_keys()
    }

    pub fn load(fname: &str) -> Result<MarisaSystemUnigramLM> {
        info!("Reading {}", fname);
        let mut trie = Trie::new();
        trie.load(fname)?;
        let Some((_, total_words)) = Self::find_from_trie(&trie, TOTAL_WORDS_KEY) else {
            bail!("Missing key for {}", TOTAL_WORDS_KEY);
        };
        let Some((_, unique_words)) = Self::find_from_trie(&trie, UNIQUE_WORDS_KEY) else {
            bail!("Missing key for {}", UNIQUE_WORDS_KEY);
        };
        Ok(MarisaSystemUnigramLM {
            trie,
            total_words: total_words as u32,
            unique_words: unique_words as u32,
        })
    }

    fn find_from_trie(trie: &Trie, word: &str) -> Option<(i32, f32)> {
        assert_ne!(word.len(), 0);

        let mut key = word.as_bytes().to_vec();
        key.push(0xff);
        let mut agent = Agent::new();
        agent.set_query_bytes(&key);

        if trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            let kanji_id = agent.key().id();

            if let Some(idx) = word.iter().position(|f| *f == b'\xff') {
                let start = idx + 1;
                if word.len() < start + 4 {
                    warn!("Malformed unigram entry: len={}, idx={}", word.len(), idx);
                    return None;
                }
                let bytes: [u8; 4] = word[start..start + 4].try_into().ok()?;
                let score = f32::from_le_bytes(bytes);
                return Some((kanji_id as i32, score));
            }
        }
        None
    }
}

impl SystemUnigramLM for MarisaSystemUnigramLM {
    fn get_cost(&self, wordcnt: u32) -> f32 {
        calc_cost(wordcnt, self.total_words, self.unique_words)
    }

    /// @return (word_id, score)。
    fn find(&self, word: &str) -> Option<(i32, f32)> {
        Self::find_from_trie(&self.trie, word)
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        let mut map = HashMap::new();
        let mut agent = Agent::new();
        agent.set_query_str("");

        while self.trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            let id = agent.key().id();

            if let Some(idx) = word.iter().position(|f| *f == b'\xff') {
                let start = idx + 1;
                if word.len() >= start + 4 {
                    let bytes: [u8; 4] = match word[start..start + 4].try_into() {
                        Ok(bytes) => bytes,
                        Err(_) => continue,
                    };
                    let word_str = String::from_utf8_lossy(&word[0..idx]);
                    let cost = f32::from_le_bytes(bytes);
                    map.insert(word_str.to_string(), (id as i32, cost));
                }
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test() {
        let named_tmpfile = NamedTempFile::new().unwrap();
        let tmpfile = named_tmpfile.path().to_str().unwrap().to_string();

        let mut builder = MarisaSystemUnigramLMBuilder::default();
        builder.add("hello", 0.4);
        builder.add("world", 0.2);
        builder.set_total_words(2);
        builder.set_unique_words(2);
        builder.save(&tmpfile).unwrap();

        let lm = MarisaSystemUnigramLM::load(&tmpfile).unwrap();
        {
            let (word_id, score) = lm.find("hello").unwrap();
            assert_eq!(word_id, 0);
            assert_eq!(score, 0.4_f32);
        }
        {
            let p = lm.find("unknown");
            assert_eq!(p, None);
        }
    }
}
