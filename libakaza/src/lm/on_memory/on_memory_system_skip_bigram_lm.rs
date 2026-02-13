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
    adjustment: RefCell<HashMap<(i32, i32), f32>>,
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
            adjustment: RefCell::new(HashMap::new()),
        }
    }

    pub fn adjust_cost(&self, word_id1: i32, word_id2: i32, delta: f32) {
        *self
            .adjustment
            .borrow_mut()
            .entry((word_id1, word_id2))
            .or_insert(0.0) += delta;
    }

    pub fn update(&self, word_id1: i32, word_id2: i32, cnt: u32) {
        self.map.borrow_mut().insert((word_id1, word_id2), cnt);
    }

    pub fn get_skip_cnt(&self, word_id1: i32, word_id2: i32) -> Option<u32> {
        self.map.borrow().get(&(word_id1, word_id2)).copied()
    }

    /// adjustment map にあるが base map にないエントリを返す。
    /// 値は default_skip_cost + adjustment。
    pub fn adjustment_only_entries(&self) -> Vec<((i32, i32), f32)> {
        let map = self.map.borrow();
        let adj = self.adjustment.borrow();
        adj.iter()
            .filter(|(key, _)| !map.contains_key(key))
            .map(|(key, &a)| (*key, self.default_skip_cost + a))
            .collect()
    }

    pub fn as_hash_map(&self) -> HashMap<(i32, i32), f32> {
        let map = self.map.borrow();
        let adj = self.adjustment.borrow();
        let mut result: HashMap<(i32, i32), f32> = map
            .iter()
            .map(|((id1, id2), cnt)| {
                let a = adj.get(&(*id1, *id2)).copied().unwrap_or(0.0);
                (
                    (*id1, *id2),
                    calc_cost(*cnt, self.total_words, self.unique_words) + a,
                )
            })
            .collect();

        // adjustment のみのエントリを追加
        for (key, &a) in adj.iter() {
            if !map.contains_key(key) {
                result.insert(*key, self.default_skip_cost + a);
            }
        }

        result
    }
}

impl SystemSkipBigramLM for OnMemorySystemSkipBigramLM {
    fn get_skip_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        let key = (word_id1, word_id2);
        let map = self.map.borrow();
        let adj = self.adjustment.borrow();
        let a = adj.get(&key).copied().unwrap_or(0.0);

        if let Some(cnt) = map.get(&key) {
            Some(calc_cost(*cnt, self.total_words, self.unique_words) + a)
        } else if a != 0.0 {
            Some(self.default_skip_cost + a)
        } else {
            None
        }
    }

    fn get_default_skip_cost(&self) -> f32 {
        self.default_skip_cost
    }
}
