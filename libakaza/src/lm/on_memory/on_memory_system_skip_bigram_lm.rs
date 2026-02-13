use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::cost::calc_cost;
use crate::lm::base::SystemSkipBigramLM;

pub struct OnMemorySystemSkipBigramLM {
    // (word_id, word_id) -> count
    map: Rc<RefCell<HashMap<(i32, i32), u32>>>,
    default_skip_cost: f32,
    pub total_words: u32,
    pub unique_words: u32,
}

impl OnMemorySystemSkipBigramLM {
    pub fn new(
        map: Rc<RefCell<HashMap<(i32, i32), u32>>>,
        default_skip_cost: f32,
        total_words: u32,
        unique_words: u32,
    ) -> Self {
        OnMemorySystemSkipBigramLM {
            map,
            default_skip_cost,
            total_words,
            unique_words,
        }
    }

    pub fn update(&self, word_id1: i32, word_id2: i32, cnt: u32) {
        self.map.borrow_mut().insert((word_id1, word_id2), cnt);
    }

    pub fn get_skip_cnt(&self, word_id1: i32, word_id2: i32) -> Option<u32> {
        self.map.borrow().get(&(word_id1, word_id2)).copied()
    }

    pub fn as_hash_map(&self) -> HashMap<(i32, i32), f32> {
        self.map
            .borrow()
            .iter()
            .map(|((id1, id2), cnt)| {
                (
                    (*id1, *id2),
                    calc_cost(*cnt, self.total_words, self.unique_words),
                )
            })
            .collect()
    }
}

impl SystemSkipBigramLM for OnMemorySystemSkipBigramLM {
    fn get_skip_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        self.map
            .borrow()
            .get(&(word_id1, word_id2))
            .map(|cnt| calc_cost(*cnt, self.total_words, self.unique_words))
    }

    fn get_default_skip_cost(&self) -> f32 {
        self.default_skip_cost
    }
}
