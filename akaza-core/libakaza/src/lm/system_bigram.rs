use crate::trie::{Trie, TrieBuilder};
use marisa_sys::Marisa;

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

    pub fn save(&self, ofname: &str) -> std::io::Result<()> {
        self.builder.save(ofname)
    }
}

struct SystemBigramLM {
    trie: Trie,
}
impl SystemBigramLM {
    /**
     * edge cost を得る。word1 と word2 は
     */
    fn get_edge_cost(word_id1: isize, word_id2: usize) -> Option<f32> {}
}
