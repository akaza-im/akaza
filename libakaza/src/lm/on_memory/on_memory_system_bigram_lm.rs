use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::lm::base::SystemBigramLM;

pub struct OnMemorySystemBigramLM {
    // (word_id, word_id) -> cost
    map: Rc<RefCell<HashMap<(i32, i32), f32>>>,
    default_edge_cost: f32,
}

impl OnMemorySystemBigramLM {
    pub fn new(map: Rc<RefCell<HashMap<(i32, i32), f32>>>, default_edge_cost: f32) -> Self {
        OnMemorySystemBigramLM {
            map,
            default_edge_cost,
        }
    }

    pub fn update(&self, word_id1: i32, word_id2: i32, cost: f32) {
        self.map.borrow_mut().insert((word_id1, word_id2), cost);
    }
}

impl SystemBigramLM for OnMemorySystemBigramLM {
    #[inline]
    fn get_default_edge_cost(&self) -> f32 {
        self.default_edge_cost
    }

    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        self.map.borrow().get(&(word_id1, word_id2)).cloned()
    }

    fn as_hash_map(&self) -> HashMap<(i32, i32), f32> {
        self.map.borrow().clone()
    }
}
