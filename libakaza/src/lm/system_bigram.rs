use anyhow::Result;
use half::f16;

use crate::trie::{Trie, TrieBuilder};

/**
 * bigram 言語モデル。
 * unigram の生成のときに得られた単語IDを利用することで、圧縮している。
 */
#[derive(Default)]
pub struct SystemBigramLMBuilder {
    builder: TrieBuilder,
}

impl SystemBigramLMBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, score: f32) {
        // edge cost 言語モデルファイルの容量を小さく保つために
        // 3 byte に ID を収めるようにする。
        // 最大でも 8,388,608 単語までになるように vocab を制限すること。
        // 現実的な線で切っても、500万単語ぐらいで十分だと思われる。

        // -rw-r--r-- 1 tokuhirom tokuhirom  28M Dec 31 23:56 stats-kytea-lm_v2_2gram.trie
        // ↓ 1MB 節約できる。
        // -rw-r--r-- 1 tokuhirom tokuhirom  27M Jan  1 02:05 stats-kytea-lm_v2_2gram.trie

        // 4+4+4=12バイト必要だったところが、3+3+4=10バイトになって、10/12=5/6 なので、
        // 本来なら 23.3 MB ぐらいまで減ってほしいところだけど、そこまではいかない。
        // TRIE 構造だからそういう感じには減らない。

        // さらに、スコアを f16 にしてみたが、あまりかわらない。
        // -rw-r--r-- 1 tokuhirom tokuhirom  27M Jan  1 02:14 stats-kytea-lm_v2_2gram.trie

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

    pub fn build(&self) -> SystemBigramLM {
        let trie = self.builder.build();
        SystemBigramLM { trie }
    }

    pub fn save(&self, ofname: &str) -> anyhow::Result<()> {
        self.builder.save(ofname)
    }
}

pub struct SystemBigramLM {
    trie: Trie,
}

impl SystemBigramLM {
    pub fn load(filename: &str) -> Result<SystemBigramLM> {
        let trie = Trie::load(filename)?;
        Ok(SystemBigramLM { trie })
    }

    pub fn num_keys(&self) -> usize {
        self.trie.num_keys()
    }

    /**
     * edge cost を得る。
     * この ID は、unigram の trie でふられたもの。
     */
    pub fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
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
        let mut builder = SystemBigramLMBuilder::default();
        builder.add(4649, 5963, 5.11_f32);
        let system_bigram_lm = builder.build();
        let got_score = system_bigram_lm.get_edge_cost(4649, 5963).unwrap();
        assert!(5.0 < got_score && got_score < 5.12);
        Ok(())
    }
}
