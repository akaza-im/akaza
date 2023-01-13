use std::collections::HashMap;

use anyhow::{bail, Result};
use log::info;

use marisa_sys::{Keyset, Marisa};

use crate::lm::base::SystemUnigramLM;

/*
   {word} # in utf-8
   0xff   # marker
   packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608(2**24/2)
   packed float  # score: 4 bytes
*/

const DEFAULT_COST_FOR_SHORT_KEY: &str = "__DEFAULT_COST_FOR_SHORT__";
const DEFAULT_COST_KEY: &str = "__DEFAULT_COST__";

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

    pub fn set_default_cost_for_short(&mut self, cost: f32) -> &mut Self {
        self.add(DEFAULT_COST_FOR_SHORT_KEY, cost);
        self
    }

    pub fn set_default_cost(&mut self, cost: f32) -> &mut Self {
        self.add(DEFAULT_COST_KEY, cost);
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
        let (_, default_cost_for_short) =
            MarisaSystemUnigramLM::find_from_trie(&marisa, DEFAULT_COST_FOR_SHORT_KEY).unwrap();
        let (_, default_cost) =
            MarisaSystemUnigramLM::find_from_trie(&marisa, DEFAULT_COST_FOR_SHORT_KEY).unwrap();
        MarisaSystemUnigramLM {
            marisa,
            default_cost_for_short,
            default_cost,
        }
    }
}

pub struct MarisaSystemUnigramLM {
    marisa: Marisa,
    default_cost_for_short: f32,
    default_cost: f32,
}

impl MarisaSystemUnigramLM {
    pub fn num_keys(&self) -> usize {
        self.marisa.num_keys()
    }

    pub fn load(fname: &str) -> Result<MarisaSystemUnigramLM> {
        info!("Reading {}", fname);
        let mut marisa = Marisa::default();
        marisa.load(fname)?;
        let Some((_, default_cost_for_short)) = Self::find_from_trie(&marisa, DEFAULT_COST_FOR_SHORT_KEY) else {
            bail!("Missing key for {}", DEFAULT_COST_FOR_SHORT_KEY);
        };
        let Some((_, default_cost)) = Self::find_from_trie(&marisa, DEFAULT_COST_FOR_SHORT_KEY) else {
            bail!("Missing key for {}", DEFAULT_COST_KEY);
        };
        Ok(MarisaSystemUnigramLM {
            marisa,
            default_cost_for_short,
            default_cost,
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
    fn get_default_cost(&self) -> f32 {
        self.default_cost
    }

    fn get_default_cost_for_short(&self) -> f32 {
        self.default_cost_for_short
    }

    /// @return (word_id, score)。
    fn find(&self, word: &str) -> Option<(i32, f32)> {
        Self::find_from_trie(&self.marisa, word)
    }

    fn as_id_map(&self) -> HashMap<String, i32> {
        let mut map = HashMap::new();
        self.marisa.predictive_search("".as_bytes(), |word, id| {
            let idx = word.iter().position(|f| *f == b'\xff').unwrap();
            let word = String::from_utf8_lossy(&word[0..idx]);
            map.insert(word.to_string(), id as i32);
            true
        });
        map
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
            .set_default_cost(20_f32)
            .set_default_cost_for_short(19_f32)
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
