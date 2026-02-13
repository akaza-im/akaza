use anyhow::{bail, Result};
use half::f16;
use log::info;

use rsmarisa::{Agent, Keyset, Trie};

use crate::lm::base::SystemSkipBigramLM;

/*
   {word1 ID}    # 3 bytes (w_{i-2})
   {word2 ID}    # 3 bytes (w_i)
   packed float  # score: 2 bytes (f16)
*/

const DEFAULT_COST_KEY: &str = "__DEFAULT_SKIP_COST__";

/// skip-bigram 言語モデルのビルダー。
/// bigram LM と同一キー形式 `[3B id1][3B id2][2B f16_score]` を使用。
pub struct MarisaSystemSkipBigramLMBuilder {
    keyset: Keyset,
}

impl Default for MarisaSystemSkipBigramLMBuilder {
    fn default() -> Self {
        Self {
            keyset: Keyset::new(),
        }
    }
}

impl MarisaSystemSkipBigramLMBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, score: f32) {
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

    pub fn set_default_skip_cost(&mut self, cost: f32) -> &mut Self {
        let key = format!("{DEFAULT_COST_KEY}\t{cost}");
        self.keyset.push_back_str(&key).unwrap();
        self
    }

    pub fn build(&mut self) -> Result<MarisaSystemSkipBigramLM> {
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        let default_skip_cost = MarisaSystemSkipBigramLM::read_default_skip_cost(&trie)?;
        Ok(MarisaSystemSkipBigramLM {
            trie,
            default_skip_cost,
        })
    }

    pub fn save(&mut self, ofname: &str) -> Result<()> {
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        trie.save(ofname)?;
        Ok(())
    }
}

pub struct MarisaSystemSkipBigramLM {
    trie: Trie,
    default_skip_cost: f32,
}

impl MarisaSystemSkipBigramLM {
    pub fn load(filename: &str) -> Result<MarisaSystemSkipBigramLM> {
        info!("Loading system-skip-bigram: {}", filename);
        let mut trie = Trie::new();
        trie.load(filename)?;
        let default_skip_cost = Self::read_default_skip_cost(&trie).unwrap_or_else(|_| {
            info!("No default skip cost in model, using fallback 10.0");
            10.0
        });
        info!("  default_skip_cost={}", default_skip_cost);
        Ok(MarisaSystemSkipBigramLM {
            trie,
            default_skip_cost,
        })
    }

    fn read_default_skip_cost(trie: &Trie) -> Result<f32> {
        let mut agent = Agent::new();
        agent.set_query_str(DEFAULT_COST_KEY);

        if trie.predictive_search(&mut agent) {
            let key = agent.key().as_str();
            if let Some((_, score)) = key.split_once('\t') {
                return Ok(score.parse::<f32>()?);
            }
        }

        bail!("Cannot read default skip cost from skip-bigram trie");
    }
}

impl SystemSkipBigramLM for MarisaSystemSkipBigramLM {
    fn get_skip_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
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

        let mut agent = Agent::new();
        agent.set_query_bytes(&key);

        if self.trie.predictive_search(&mut agent) {
            let keyword = agent.key().as_bytes();
            if keyword.len() < 2 {
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

    fn get_default_skip_cost(&self) -> f32 {
        self.default_skip_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_lookup() -> anyhow::Result<()> {
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        builder.add(100, 200, 3.5);
        builder.add(100, 300, 4.0);
        builder.set_default_skip_cost(10.0);
        let lm = builder.build()?;

        let cost = lm.get_skip_cost(100, 200).unwrap();
        assert!(3.4 < cost && cost < 3.6);

        let cost = lm.get_skip_cost(100, 300).unwrap();
        assert!(3.9 < cost && cost < 4.1);

        assert!(lm.get_skip_cost(999, 888).is_none());
        assert!((lm.get_default_skip_cost() - 10.0).abs() < f32::EPSILON);
        Ok(())
    }

    #[test]
    fn default_cost_fallback() -> anyhow::Result<()> {
        // デフォルトコスト未設定の古いモデル → フォールバック値 10.0
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        builder.add(1, 2, 5.0);
        // set_default_skip_cost を呼ばない
        let mut trie = Trie::new();
        trie.build(&mut builder.keyset, 0);
        let result = MarisaSystemSkipBigramLM::read_default_skip_cost(&trie);
        assert!(result.is_err());
        Ok(())
    }
}
