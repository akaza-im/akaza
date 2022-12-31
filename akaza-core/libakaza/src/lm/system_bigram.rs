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
        let mut key: Vec<u8> = Vec::new();
        key.extend(word_id1.to_le_bytes());
        key.extend(word_id2.to_le_bytes());
        key.extend(score.to_le_bytes());
        self.builder.add(key);
    }

    pub fn build(&self) -> SystemBigramLM {
        let trie = self.builder.build();
        SystemBigramLM { trie }
    }

    pub fn save(&self, ofname: &str) -> std::io::Result<()> {
        self.builder.save(ofname)
    }
}

pub struct SystemBigramLM {
    trie: Trie,
}

impl SystemBigramLM {
    pub fn load(filename: &str) -> Result<SystemBigramLM, anyhow::Error> {
        let trie = Trie::load(filename)?;
        Ok(SystemBigramLM { trie })
    }

    pub fn num_keys(&self) -> usize {
        self.trie.num_keys()
    }

    /**
     * edge cost を得る。word1 と word2 は
     */
    pub fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        let key = [word_id1.to_le_bytes(), word_id2.to_le_bytes()].concat();
        let got = self.trie.predictive_search(key);
        let Some(result) = got.first() else {
            return None;
        };
        let last4: [u8; 4] = result.keyword[result.keyword.len() - 4..result.keyword.len()]
            .try_into()
            .unwrap();
        let score = f32::from_le_bytes(last4);
        Some(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_load() {
        let mut builder = SystemBigramLMBuilder::default();
        builder.add(4649, 5963, 5.11_f32);
        let trie = builder.build();
        let got_score = trie.get_edge_cost(4649, 5963).unwrap();
        assert_eq!(got_score, 5.11_f32);
    }
}
