use std::collections::HashMap;

use anyhow::{bail, Result};
use log::info;

use marisa_sys::{Keyset, Marisa};

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

    pub fn keyset(&self) -> Keyset {
        let mut keyset = Keyset::default();
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
            keyset.push_back(key.as_slice());
        }
        keyset
    }

    pub fn set_total_words(&mut self, total_words: u32) -> &mut Self {
        self.add(TOTAL_WORDS_KEY, total_words as f32);
        self
    }

    pub fn set_unique_words(&mut self, unique_words: u32) -> &mut Self {
        self.add(UNIQUE_WORDS_KEY, unique_words as f32);
        self
    }

    pub fn save(&self, fname: &str) -> Result<()> {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset());
        marisa.save(fname)?;
        Ok(())
    }

    pub fn build(&self) -> MarisaSystemUnigramLM {
        let mut marisa = Marisa::default();
        marisa.build(&self.keyset());
        let (_, total_words) =
            MarisaSystemUnigramLM::find_from_trie(&marisa, TOTAL_WORDS_KEY).unwrap();
        let (_, unique_words) =
            MarisaSystemUnigramLM::find_from_trie(&marisa, UNIQUE_WORDS_KEY).unwrap();
        MarisaSystemUnigramLM {
            marisa,
            total_words: total_words as u32,
            unique_words: unique_words as u32,
        }
    }
}

pub struct MarisaSystemUnigramLM {
    marisa: Marisa,
    total_words: u32,
    unique_words: u32,
}

impl MarisaSystemUnigramLM {
    pub fn num_keys(&self) -> usize {
        self.marisa.num_keys()
    }

    pub fn load(fname: &str) -> Result<MarisaSystemUnigramLM> {
        info!("Reading {}", fname);
        let mut marisa = Marisa::default();
        marisa.load(fname)?;
        let Some((_, total_words)) = Self::find_from_trie(&marisa, TOTAL_WORDS_KEY) else {
            bail!("Missing key for {}", TOTAL_WORDS_KEY);
        };
        let Some((_, unique_words)) = Self::find_from_trie(&marisa, UNIQUE_WORDS_KEY) else {
            bail!("Missing key for {}", UNIQUE_WORDS_KEY);
        };
        Ok(MarisaSystemUnigramLM {
            marisa,
            total_words: total_words as u32,
            unique_words: unique_words as u32,
        })
    }

    fn find_from_trie(marisa: &Marisa, word: &str) -> Option<(i32, f32)> {
        assert_ne!(word.len(), 0);

        let key = [word.as_bytes(), b"\xff"].concat();
        let mut kanji_id: usize = usize::MAX;
        let mut score = f32::MAX;
        marisa.predictive_search(key.as_slice(), |word, id| {
            kanji_id = id;

            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
            score = f32::from_le_bytes(bytes);
            false
        });
        if kanji_id != usize::MAX {
            Some((kanji_id as i32, score))
        } else {
            None
        }
    }
}

impl SystemUnigramLM for MarisaSystemUnigramLM {
    fn get_cost(&self, wordcnt: u32) -> f32 {
        calc_cost(wordcnt, self.total_words, self.unique_words)
    }

    /// @return (word_id, score)。
    fn find(&self, word: &str) -> Option<(i32, f32)> {
        Self::find_from_trie(&self.marisa, word)
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        let mut map = HashMap::new();
        self.marisa.predictive_search("".as_bytes(), |word, id| {
            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let bytes: [u8; 4] = word[idx + 1..idx + 1 + 4].try_into().unwrap();
            let word = String::from_utf8_lossy(&word[0..idx]);
            let cost = f32::from_le_bytes(bytes);
            map.insert(word.to_string(), (id as i32, cost));
            true
        });
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
        builder
            .set_total_words(2)
            .set_unique_words(2)
            .save(&tmpfile)
            .unwrap();

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
