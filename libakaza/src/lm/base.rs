use std::collections::HashMap;

pub trait SystemBigramLM {
    fn get_default_edge_cost(&self) -> f32;
    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32>;
    fn as_hash_map(&self) -> HashMap<(i32, i32), f32>;
}

pub trait SystemUnigramLM {
    fn get_cost(&self, wordcnt: u32) -> f32;

    fn find(&self, word: &str) -> Option<(i32, f32)>;
    fn as_hash_map(&self) -> HashMap<String, (i32, f32)>;
}

pub trait SystemSkipBigramLM {
    /// skip-bigram コストを返す（w_{i-2} と w_i のペア）。
    /// 見つからなければ None。
    fn get_skip_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32>;

    /// ペアが見つからなかった場合のデフォルトコスト。
    fn get_default_skip_cost(&self) -> f32;
}
