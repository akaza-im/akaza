use std::collections::btree_map::BTreeMap;
use std::rc::Rc;

use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::SegmentationResult;
use crate::graph::word_node::WordNode;
use crate::kana_kanji_dict::KanaKanjiDict;
use crate::lm::system_bigram::SystemBigramLM;
use crate::lm::system_unigram_lm::SystemUnigramLM;
use crate::user_side_data::user_data::UserData;

pub struct GraphBuilder {
    system_kana_kanji_dict: KanaKanjiDict,
    system_single_term_dict: KanaKanjiDict,
    user_data: Rc<UserData>,
    system_unigram_lm: Rc<SystemUnigramLM>,
    system_bigram_lm: Rc<SystemBigramLM>,
}

impl GraphBuilder {
    pub fn new(
        system_kana_kanji_dict: KanaKanjiDict,
        system_single_term_dict: KanaKanjiDict,
        user_data: Rc<UserData>,
        system_unigram_lm: Rc<SystemUnigramLM>,
        system_bigram_lm: Rc<SystemBigramLM>,
    ) -> GraphBuilder {
        GraphBuilder {
            system_kana_kanji_dict,
            system_single_term_dict,
            user_data,
            system_unigram_lm,
            system_bigram_lm,
        }
    }

    pub fn construct(&self, yomi: &str, words_ends_at: SegmentationResult) -> LatticeGraph {
        // このグラフのインデクスは単語の終了位置。
        let mut graph: BTreeMap<i32, Vec<WordNode>> = BTreeMap::new();
        graph.insert(0, vec![WordNode::create_bos()]);
        graph.insert(
            (yomi.len() + 1) as i32,
            vec![WordNode::create_eos(yomi.len() as i32)],
        );

        for (end_pos, segmented_yomis) in words_ends_at.iter() {
            for segmented_yomi in segmented_yomis {
                let vec = graph.entry(*end_pos as i32).or_default();

                // ひらがなそのものもエントリーとして登録しておく。
                // TODO これが結果として重複につながってそう。辞書にないときだけ入れるようにしたほうが良いかも。
                let node = WordNode::new(
                    (end_pos - segmented_yomi.len()) as i32,
                    segmented_yomi,
                    segmented_yomi,
                );
                vec.push(node);

                // 漢字に変換した結果もあれば insert する。
                if let Some(kanjis) = self.system_kana_kanji_dict.find(segmented_yomi) {
                    for kanji in kanjis {
                        let node = WordNode::new(
                            (end_pos - segmented_yomi.len()) as i32,
                            &kanji,
                            segmented_yomi,
                        );
                        vec.push(node);
                    }
                }

                // 変換範囲が全体になっていれば single term 辞書を利用する。
                if segmented_yomi == yomi {
                    if let Some(surfaces) = self.system_single_term_dict.find(yomi) {
                        for surface in surfaces {
                            let node = WordNode::new(
                                (end_pos - segmented_yomi.len()) as i32,
                                &surface,
                                segmented_yomi,
                            );
                            vec.push(node);
                        }
                    }
                }
            }
        }
        LatticeGraph {
            graph,
            yomi: yomi.to_string(),
            user_data: self.user_data.clone(),
            system_unigram_lm: self.system_unigram_lm.clone(),
            system_bigram_lm: self.system_bigram_lm.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kana_kanji_dict::KanaKanjiDictBuilder;
    use crate::lm::system_bigram::SystemBigramLMBuilder;
    use crate::lm::system_unigram_lm::SystemUnigramLMBuilder;

    use super::*;

    #[test]
    fn test_single_term() {
        let graph_builder = GraphBuilder::new(
            KanaKanjiDict::default(),
            KanaKanjiDictBuilder::default().add("すし", "🍣").build(),
            Rc::new(UserData::default()),
            Rc::new(SystemUnigramLMBuilder::default().build()),
            Rc::new(SystemBigramLMBuilder::default().build()),
        );
        let yomi = "すし";
        let got = graph_builder.construct(
            yomi,
            SegmentationResult::new(BTreeMap::from([(6, vec!["すし".to_string()])])),
        );
        let nodes = got.node_list(6).unwrap();
        let got_surfaces: Vec<String> = nodes.iter().map(|f| f.kanji.to_string()).collect();
        assert_eq!(got_surfaces, vec!["すし".to_string(), "🍣".to_string()]);
    }
}
