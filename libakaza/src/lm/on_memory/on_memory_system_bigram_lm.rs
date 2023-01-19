use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::cost::calc_cost;
use crate::lm::base::SystemBigramLM;

pub struct OnMemorySystemBigramLM {
    // (word_id, word_id) -> cost
    map: Rc<RefCell<HashMap<(i32, i32), u32>>>,
    default_edge_cost: f32,
    pub c: u32,
    pub v: u32,
}

impl OnMemorySystemBigramLM {
    pub fn new(
        map: Rc<RefCell<HashMap<(i32, i32), u32>>>,
        default_edge_cost: f32,
        c: u32,
        v: u32,
    ) -> Self {
        OnMemorySystemBigramLM {
            map,
            default_edge_cost,
            c,
            v,
        }
    }

    pub fn update(&self, word_id1: i32, word_id2: i32, cnt: u32) {
        self.map.borrow_mut().insert((word_id1, word_id2), cnt);
    }

    pub fn get_edge_cnt(&self, word_id1: i32, word_id2: i32) -> Option<u32> {
        self.map.borrow().get(&(word_id1, word_id2)).copied()
    }
}

impl SystemBigramLM for OnMemorySystemBigramLM {
    #[inline]
    fn get_default_edge_cost(&self) -> f32 {
        self.default_edge_cost
    }

    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        self.map
            .borrow()
            .get(&(word_id1, word_id2))
            .map(|f| calc_cost(*f, self.c, self.v))
    }

    fn as_hash_map(&self) -> HashMap<(i32, i32), f32> {
        self.map
            .borrow()
            .iter()
            .map(|((id1, id2), cnt)| ((*id1, *id2), calc_cost(*cnt, self.c, self.v)))
            .collect()
    }
}
