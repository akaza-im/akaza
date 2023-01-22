use crate::cost::calc_cost;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::lm::base::SystemUnigramLM;

pub struct OnMemorySystemUnigramLM {
    // word -> (word_id, cost)
    map: Rc<RefCell<HashMap<String, (i32, u32)>>>,
    pub default_cost: f32,
    pub default_cost_for_short: f32,
    pub total_words: u32,
    pub unique_words: u32,
}

impl OnMemorySystemUnigramLM {
    pub fn new(
        map: Rc<RefCell<HashMap<String, (i32, u32)>>>,
        default_cost: f32,
        default_cost_for_short: f32,
        c: u32,
        v: u32,
    ) -> Self {
        OnMemorySystemUnigramLM {
            map,
            default_cost,
            default_cost_for_short,
            total_words: c,
            unique_words: v,
        }
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
    fn get_default_cost(&self) -> f32 {
        self.default_cost
    }

    fn get_default_cost_for_short(&self) -> f32 {
        self.default_cost_for_short
    }

    fn find(&self, word: &str) -> Option<(i32, f32)> {
        self.map
            .borrow()
            .get(word)
            .map(|(id, cnt)| (*id, calc_cost(*cnt, self.total_words, self.unique_words)))
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        self.map
            .borrow()
            .iter()
            .map(|(key, (id, cnt))| {
                (
                    key.to_string(),
                    (*id, calc_cost(*cnt, self.total_words, self.unique_words)),
                )
            })
            .collect()
    }
}
