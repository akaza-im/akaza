use std::collections::HashMap;

pub trait SystemBigramLM {
    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32>;
}

pub trait SystemUnigramLM {
    fn find(&self, word: &str) -> Option<(i32, f32)>;
    fn as_id_map(&self) -> HashMap<String, i32>;
}
