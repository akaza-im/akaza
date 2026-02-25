use std::cell::RefCell;

use rustc_hash::FxHashMap;

use anyhow::{bail, Result};
use half::f16;
use log::{info, warn};

use rsmarisa::{Agent, Keyset, Trie};

use crate::lm::base::SystemBigramLM;
use crate::lm::model_metadata::{add_metadata_to_keyset, read_metadata_from_trie, ModelMetadata};

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
pub struct MarisaSystemBigramLMBuilder {
    keyset: Keyset,
    metadata: ModelMetadata,
}

impl Default for MarisaSystemBigramLMBuilder {
    fn default() -> Self {
        Self {
            keyset: Keyset::new(),
            metadata: ModelMetadata::default(),
        }
    }
}

impl MarisaSystemBigramLMBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, score: f32) {
        // edge cost 言語モデルファイルの容量を小さく保つために
        // 3 byte に ID を収めるようにする。
        // 最大でも 8,388,608 単語までになるように vocab を制限すること。
        // 現実的な線で切っても、500万単語ぐらいで十分だと思われる。

        // -rw-r--r-- 1 tokuhirom tokuhirom  28M Dec 31 23:56 bigram.model
        // ↓ 1MB 節約できる。
        // -rw-r--r-- 1 tokuhirom tokuhirom  27M Jan  1 02:05 bigram.model

        // 4+4+4=12バイト必要だったところが、3+3+4=10バイトになって、10/12=5/6 なので、
        // 本来なら 23.3 MB ぐらいまで減ってほしいところだけど、そこまではいかない。
        // TRIE 構造だからそういう感じには減らない。

        // さらに、スコアを f16 にしてみたが、あまりかわらない。
        // -rw-r--r-- 1 tokuhirom tokuhirom  27M Jan  1 02:14 bigram.model

        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();

        assert_eq!(id1_bytes[3], 0);
        assert_eq!(id2_bytes[3], 0);

        let mut key: Vec<u8> = Vec::new();
        key.extend(id1_bytes[0..3].iter());
        key.extend(id2_bytes[0..3].iter());
        key.extend(f16::from_f32(score).to_le_bytes());
        self.keyset.push_back_bytes(&key, 1.0).unwrap();
    }

    pub fn set_default_edge_cost(&mut self, score: f32) -> &mut Self {
        let key = format!("{DEFAULT_COST_KEY}\t{score}");
        self.keyset.push_back_str(&key).unwrap();
        self
    }

    pub fn set_metadata(&mut self, metadata: ModelMetadata) -> &mut Self {
        self.metadata = metadata;
        self
    }

    pub fn build(&mut self) -> Result<MarisaSystemBigramLM> {
        add_metadata_to_keyset(&mut self.keyset, &self.metadata);
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        let default_edge_cost = MarisaSystemBigramLM::read_default_edge_cost(&trie)?;
        Ok(MarisaSystemBigramLM {
            trie,
            default_edge_cost,
            agent: RefCell::new(Agent::new()),
        })
    }

    pub fn save(&mut self, ofname: &str) -> Result<()> {
        add_metadata_to_keyset(&mut self.keyset, &self.metadata);
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        trie.save(ofname)?;
        Ok(())
    }
}

pub struct MarisaSystemBigramLM {
    trie: Trie,
    default_edge_cost: f32,
    /// 検索用 Agent の再利用（毎回のアロケーションを避ける）
    agent: RefCell<Agent>,
}

impl MarisaSystemBigramLM {
    pub fn load(filename: &str) -> Result<MarisaSystemBigramLM> {
        info!("Loading system-bigram: {}", filename);
        let mut trie = Trie::new();
        trie.load(filename)?;
        let default_edge_cost = Self::read_default_edge_cost(&trie)?;
        Ok(MarisaSystemBigramLM {
            trie,
            default_edge_cost,
            agent: RefCell::new(Agent::new()),
        })
    }

    pub fn num_keys(&self) -> usize {
        self.trie.num_keys()
    }

    /// bigram に (word_id1, word_id2) のエントリが含まれているか検索する。
    /// @return Some(score) if found, None otherwise.
    pub fn lookup(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        self.get_edge_cost(word_id1, word_id2)
    }

    pub fn metadata(&self) -> ModelMetadata {
        read_metadata_from_trie(&self.trie)
    }

    fn read_default_edge_cost(trie: &Trie) -> Result<f32> {
        let mut agent = Agent::new();
        agent.set_query_str(DEFAULT_COST_KEY);

        if trie.predictive_search(&mut agent) {
            let key = agent.key().as_str();
            if let Some((_, score)) = key.split_once('\t') {
                return Ok(score.parse::<f32>()?);
            }
        }

        bail!("Cannot read default cost from bigram-trie");
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
        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();
        let key: [u8; 6] = [
            id1_bytes[0],
            id1_bytes[1],
            id1_bytes[2],
            id2_bytes[0],
            id2_bytes[1],
            id2_bytes[2],
        ];

        let mut agent = self.agent.borrow_mut();
        agent.set_query_bytes(&key);

        if self.trie.predictive_search(&mut agent) {
            let keyword = agent.key().as_bytes();
            if keyword.len() < 2 {
                warn!("Malformed bigram entry: len={}", keyword.len());
                return None;
            }
            let last2: [u8; 2] = match keyword[keyword.len() - 2..keyword.len()].try_into() {
                Ok(bytes) => bytes,
                Err(_) => return None,
            };
            let score: f16 = f16::from_le_bytes(last2);
            return Some(score.to_f32());
        }

        None
    }

    fn as_hash_map(&self) -> FxHashMap<(i32, i32), f32> {
        let mut map: FxHashMap<(i32, i32), f32> = FxHashMap::default();
        let mut agent = Agent::new();
        agent.set_query_str("");

        while self.trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            if word.len() == 8 {
                let word_id1 = i32::from_le_bytes([word[0], word[1], word[2], 0]);
                let word_id2 = i32::from_le_bytes([word[3], word[4], word[5], 0]);
                let cost = f16::from_le_bytes([word[6], word[7]]).to_f32();
                map.insert((word_id1, word_id2), cost);
            }
        }
        map
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

        let map = system_bigram_lm.as_hash_map();
        assert!(map.contains_key(&(4649, 5963)));
        let g = *map.get(&(4649, 5963)).unwrap();
        assert!(5.10_f32 < g && g < 5.12_f32);

        Ok(())
    }
}
