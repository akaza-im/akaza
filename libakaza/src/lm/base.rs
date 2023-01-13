use std::collections::HashMap;

pub trait SystemBigramLM {
    fn get_default_edge_cost(&self) -> f32;
    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32>;
}

pub trait SystemUnigramLM {
    fn get_default_cost(&self) -> f32;
    fn get_default_cost_for_short(&self) -> f32;

    fn find(&self, word: &str) -> Option<(i32, f32)>;
    fn as_hash_map(&self) -> HashMap<String, (i32, f32)>;
}
