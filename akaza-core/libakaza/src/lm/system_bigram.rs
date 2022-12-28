use crate::trie::TrieBuilder;

pub struct SystemBigramLMBuilder {
    builder: TrieBuilder,
}

impl SystemBigramLMBuilder {
    pub unsafe fn new() -> SystemBigramLMBuilder {
        SystemBigramLMBuilder {
            builder: TrieBuilder::new(),
        }
    }

    pub unsafe fn add(&self, word_id1: i32, word_id2: i32, score: f32) {
        let mut key: Vec<u8> = Vec::new();
        key.extend(word_id1.to_le_bytes());
        key.extend(word_id2.to_le_bytes());
        key.extend(score.to_le_bytes());
        self.builder.add(key);
    }

    pub unsafe fn save(&self, ofname: &String) -> std::io::Result<()> {
        return self.builder.save(ofname);
    }
}
