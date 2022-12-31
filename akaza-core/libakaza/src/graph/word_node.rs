use std::collections::btree_map::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use log::trace;

use crate::kana_kanji_dict::KanaKanjiDict;
use crate::kana_trie::KanaTrie;
use crate::lm::system_unigram_lm::SystemUnigramLM;
use crate::user_side_data::user_data::UserData;

pub struct WordNode {
    pub start_pos: i32,
    /// 漢字
    pub kanji: String,
    /// 読み仮名
    pub yomi: String,
    pub cost: f32,
}

impl Hash for WordNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start_pos.hash(state);
        self.kanji.hash(state);
        self.yomi.hash(state);
        u32::from_le_bytes(self.cost.to_le_bytes()).hash(state);
    }
}

impl PartialEq<Self> for WordNode {
    fn eq(&self, other: &Self) -> bool {
        self.start_pos == other.start_pos
            && self.kanji == other.kanji
            && self.yomi == other.yomi
            && self.cost == other.cost
    }
}

impl Eq for WordNode {}

impl WordNode {
    pub(crate) fn create_bos() -> WordNode {
        WordNode {
            start_pos: 0,
            kanji: "__BOS__".to_string(),
            yomi: "__BOS__".to_string(),
            cost: 0_f32,
        }
    }
    pub(crate) fn create_eos(start_pos: i32) -> WordNode {
        WordNode {
            start_pos,
            kanji: "__EOS__".to_string(),
            yomi: "__EOS__".to_string(),
            cost: 0_f32,
        }
    }
    pub(crate) fn new(start_pos: i32, kanji: &str, yomi: &str) -> WordNode {
        WordNode {
            start_pos,
            kanji: kanji.to_string(),
            yomi: yomi.to_string(),
            cost: 0_f32,
        }
    }
}

impl Display for WordNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.kanji, self.yomi)
    }
}
