use anyhow::{bail, Result};
use half::f16;
use log::info;

use crate::lm::base::SystemBigramLM;
use crate::trie::{Trie, TrieBuilder};

/*
   {word1 ID}    # 3 bytes
   {word2 ID}    # 3 bytes
   packed float  # score: 4 bytes
*/

const DEFAULT_COST_KEY: &str = "__DEFAULT_EDGE_COST__";

/**
 * bigram 言語モデル。
 * unigram の生成のときに得られた単語IDを利用することで、圧縮している。
 */
#[derive(Default)]
pub struct MarisaSystemBigramLMBuilder {
    builder: TrieBuilder,
}

impl MarisaSystemBigramLMBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, score: f32) {
        // edge cost 言語モデルファイルの容量を小さく保つために
        // 3 byte に ID を収めるようにする。
        // 最大でも 8,388,608 単語までになるように vocab を制限すること。
        // 現実的な線で切っても、500万単語ぐらいで十分だと思われる。

        // -rw-r--r-- 1 tokuhirom tokuhirom  28M Dec 31 23:56 stats-vibrato-bigram.trie
        // ↓ 1MB 節約できる。
        // -rw-r--r-- 1 tokuhirom tokuhirom  27M Jan  1 02:05 stats-vibrato-bigram.trie

        // 4+4+4=12バイト必要だったところが、3+3+4=10バイトになって、10/12=5/6 なので、
        // 本来なら 23.3 MB ぐらいまで減ってほしいところだけど、そこまではいかない。
        // TRIE 構造だからそういう感じには減らない。

        // さらに、スコアを f16 にしてみたが、あまりかわらない。
        // -rw-r--r-- 1 tokuhirom tokuhirom  27M Jan  1 02:14 stats-vibrato-bigram.trie

        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();

        assert_eq!(id1_bytes[3], 0);
        assert_eq!(id2_bytes[3], 0);

        let mut key: Vec<u8> = Vec::new();
        key.extend(id1_bytes[0..3].iter());
        key.extend(id2_bytes[0..3].iter());
        key.extend(f16::from_f32(score).to_le_bytes());
        self.builder.add(key);
    }

    pub fn set_default_edge_cost(&mut self, score: f32) -> &mut Self {
        let key = format!("{}\t{}", DEFAULT_COST_KEY, score);
        self.builder.add(key.as_bytes().to_vec());
        self
    }

    pub fn build(&self) -> Result<MarisaSystemBigramLM> {
        let trie = self.builder.build();
        let default_edge_cost = MarisaSystemBigramLM::read_default_edge_cost(&trie)?;
        Ok(MarisaSystemBigramLM {
            trie,
            default_edge_cost,
        })
    }

    pub fn save(&self, ofname: &str) -> anyhow::Result<()> {
        self.builder.save(ofname)
    }
}

pub struct MarisaSystemBigramLM {
    trie: Trie,
    default_edge_cost: f32,
}

impl MarisaSystemBigramLM {
    pub fn load(filename: &str) -> Result<MarisaSystemBigramLM> {
        info!("Loading system-bigram: {}", filename);
        let trie = Trie::load(filename)?;
        let default_edge_cost = Self::read_default_edge_cost(&trie);
        Ok(MarisaSystemBigramLM {
            trie,
            default_edge_cost: default_edge_cost?,
        })
    }

    pub fn num_keys(&self) -> usize {
        self.trie.num_keys()
    }

    fn read_default_edge_cost(trie: &Trie) -> Result<f32> {
        let mut keys: Vec<Vec<u8>> = Vec::new();
        trie.marisa
            .predictive_search(DEFAULT_COST_KEY.as_bytes(), |key, _| {
                keys.push(key.to_vec());
                false
            });

        let Some(key) = keys.get(0) else {
            bail!("Cannot read default cost from trie");
        };

        let key = String::from_utf8_lossy(key);
        if let Some((_, score)) = key.split_once('\t') {
            Ok(score.parse::<f32>()?)
        } else {
            bail!("Cannot parse default edge cost from trie");
        }
    }
}

impl SystemBigramLM for MarisaSystemBigramLM {
    fn get_default_edge_cost(&self) -> f32 {
        self.default_edge_cost
    }

    /**
     * edge cost を得る。
     * この ID は、unigram の trie でふられたもの。
     */
    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        let mut key: Vec<u8> = Vec::new();
        key.extend(word_id1.to_le_bytes()[0..3].iter());
        key.extend(word_id2.to_le_bytes()[0..3].iter());
        let got = self.trie.predictive_search(key);
        let Some(result) = got.first() else {
            return None;
        };
        let last2: [u8; 2] = result.keyword[result.keyword.len() - 2..result.keyword.len()]
            .try_into()
            .unwrap();
        let score: f16 = f16::from_le_bytes(last2);
        Some(score.to_f32())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_load() -> anyhow::Result<()> {
        let mut builder = MarisaSystemBigramLMBuilder::default();
        builder.set_default_edge_cost(20_f32);
        builder.add(4649, 5963, 5.11_f32);
        let system_bigram_lm = builder.build()?;
        let got_score = system_bigram_lm.get_edge_cost(4649, 5963).unwrap();
        assert!(5.0 < got_score && got_score < 5.12);
        Ok(())
    }
}
