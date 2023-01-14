use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::lm::base::SystemUnigramLM;

pub struct OnMemorySystemUnigramLM {
    // word -> (word_id, cost)
    map: Rc<RefCell<HashMap<String, (i32, f32)>>>,
    pub default_cost: f32,
    pub default_cost_for_short: f32,
}

impl OnMemorySystemUnigramLM {
    pub fn new(
        map: Rc<RefCell<HashMap<String, (i32, f32)>>>,
        default_cost: f32,
        default_cost_for_short: f32,
    ) -> Self {
        OnMemorySystemUnigramLM {
            map,
            default_cost,
            default_cost_for_short,
        }
    }

    pub fn update(&self, word: &str, cost: f32) {
        let Some((word_id, _)) = self.find(word) else {
            // 登録されてない単語は無視。
            return;
        };

        self.map
            .borrow_mut()
            .insert(word.to_string(), (word_id, cost));
    }

    pub fn reverse_lookup(&self, word_id: i32) -> Option<String> {
        self.map
            .borrow()
            .iter()
            .filter(|(_, (id, _))| *id == word_id)
            .map(|(key, (_, _))| key.clone())
            .next()
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
        self.map.borrow().get(word).copied()
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        self.map.borrow().clone()
    }
}
