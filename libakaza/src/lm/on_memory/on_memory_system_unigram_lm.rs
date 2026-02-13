use crate::cost::calc_cost;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::lm::base::SystemUnigramLM;

pub struct OnMemorySystemUnigramLM {
    // word -> (word_id, cost)
    map: Rc<RefCell<HashMap<String, (i32, u32)>>>,
    pub total_words: u32,
    pub unique_words: u32,
    adjustment: RefCell<HashMap<String, f32>>,
}

impl OnMemorySystemUnigramLM {
    pub fn new(
        map: Rc<RefCell<HashMap<String, (i32, u32)>>>,
        total_words: u32,
        unique_words: u32,
    ) -> Self {
        OnMemorySystemUnigramLM {
            map,
            total_words,
            unique_words,
            adjustment: RefCell::new(HashMap::new()),
        }
    }

    pub fn adjust_cost(&self, word: &str, delta: f32) {
        *self
            .adjustment
            .borrow_mut()
            .entry(word.to_string())
            .or_insert(0.0) += delta;
    }

    pub fn update(&self, word: &str, cnt: u32) {
        let Some((word_id, _)) = self.find(word) else {
            // 登録されてない単語は無視。
            return;
        };

        self.map
            .borrow_mut()
            .insert(word.to_string(), (word_id, cnt));
    }

    pub fn reverse_lookup(&self, word_id: i32) -> Option<String> {
        self.map
            .borrow()
            .iter()
            .filter(|(_, (id, _))| *id == word_id)
            .map(|(key, (_, _))| key.clone())
            .next()
    }

    pub fn find_cnt(&self, word: &str) -> Option<(i32, u32)> {
        self.map.borrow().get(word).copied()
    }
}

impl SystemUnigramLM for OnMemorySystemUnigramLM {
    fn get_cost(&self, wordcnt: u32) -> f32 {
        calc_cost(wordcnt, self.total_words, self.unique_words)
    }

    fn find(&self, word: &str) -> Option<(i32, f32)> {
        self.map.borrow().get(word).map(|(id, cnt)| {
            let base_cost = calc_cost(*cnt, self.total_words, self.unique_words);
            let adj = self.adjustment.borrow().get(word).copied().unwrap_or(0.0);
            (*id, base_cost + adj)
        })
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        let adj = self.adjustment.borrow();
        self.map
            .borrow()
            .iter()
            .map(|(key, (id, cnt))| {
                let base_cost = calc_cost(*cnt, self.total_words, self.unique_words);
                let a = adj.get(key).copied().unwrap_or(0.0);
                (key.to_string(), (*id, base_cost + a))
            })
            .collect()
    }
}
