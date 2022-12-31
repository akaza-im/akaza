use std::collections::btree_map::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::word_node::WordNode;
use log::trace;

use crate::kana_kanji_dict::KanaKanjiDict;
use crate::kana_trie::KanaTrie;
use crate::lm::system_unigram_lm::SystemUnigramLM;
use crate::user_side_data::user_data::UserData;

pub struct GraphBuilder {
    system_kana_kanji_dict: KanaKanjiDict,
    user_data: Rc<UserData>,
    system_unigram_lm: Rc<SystemUnigramLM>,
}

impl GraphBuilder {
    pub fn new(
        system_kana_kanji_dict: KanaKanjiDict,
        user_data: Rc<UserData>,
        system_unigram_lm: Rc<SystemUnigramLM>,
    ) -> GraphBuilder {
        GraphBuilder {
            system_kana_kanji_dict,
            user_data,
            system_unigram_lm,
        }
    }

    pub fn construct(
        &self,
        yomi: &String,
        words_ends_at: HashMap<usize, Vec<String>>,
    ) -> LatticeGraph {
        // このグラフのインデクスは単語の終了位置。
        let mut graph: BTreeMap<i32, Vec<WordNode>> = BTreeMap::new();
        graph.insert(0, vec![WordNode::create_bos()]);
        graph.insert(
            (yomi.len() + 1) as i32,
            vec![WordNode::create_eos(yomi.len() as i32)],
        );

        for (end_pos, yomis) in words_ends_at {
            for yomi in yomis {
                let vec = graph.entry(end_pos as i32).or_default();

                // ひらがなそのものもエントリーとして登録しておく。
                let node = WordNode::new((end_pos - yomi.len()) as i32, &yomi, &yomi);
                vec.push(node);

                // 漢字に変換した結果もあれば insert する。
                if let Some(kanjis) = self.system_kana_kanji_dict.find(&yomi) {
                    for kanji in kanjis {
                        let node = WordNode::new((end_pos - yomi.len()) as i32, &kanji, &yomi);
                        vec.push(node);
                    }
                }
            }
        }
        LatticeGraph {
            graph,
            user_data: self.user_data.clone(),
            system_unigram_lm: self.system_unigram_lm.clone(),
        }
    }
}
