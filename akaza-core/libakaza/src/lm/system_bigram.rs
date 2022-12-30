use crate::trie::TrieBuilder;

/**
 * bigram 言語モデル。
 * unigram の生成のときに得られた単語IDを利用することで、圧縮している。
 */
pub struct SystemBigramLMBuilder {
    builder: TrieBuilder,
}

impl SystemBigramLMBuilder {
    pub fn new() -> SystemBigramLMBuilder {
        SystemBigramLMBuilder {
            builder: TrieBuilder::new(),
        }
    }

    pub fn add(&self, word_id1: i32, word_id2: i32, score: f32) {
        let mut key: Vec<u8> = Vec::new();
        key.extend(word_id1.to_le_bytes());
        key.extend(word_id2.to_le_bytes());
        key.extend(score.to_le_bytes());
        self.builder.add(key);
    }

    pub fn save(&self, ofname: &String) -> std::io::Result<()> {
        return self.builder.save(ofname);
    }
}
