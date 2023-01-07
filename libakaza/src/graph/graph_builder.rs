use std::collections::btree_map::BTreeMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use kelp::{hira2kata, ConvOption};

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
    user_data: Arc<Mutex<UserData>>,
    system_unigram_lm: Rc<SystemUnigramLM>,
    system_bigram_lm: Rc<SystemBigramLM>,
    default_unigram_score_for_long: f32,
    default_unigram_score_for_short: f32,
}

impl GraphBuilder {
    pub fn new(
        system_kana_kanji_dict: KanaKanjiDict,
        system_single_term_dict: KanaKanjiDict,
        user_data: Arc<Mutex<UserData>>,
        system_unigram_lm: Rc<SystemUnigramLM>,
        system_bigram_lm: Rc<SystemBigramLM>,
        default_unigram_score_for_short: f32,
        default_unigram_score_for_long: f32,
    ) -> GraphBuilder {
        GraphBuilder {
            system_kana_kanji_dict,
            system_single_term_dict,
            user_data,
            system_unigram_lm,
            system_bigram_lm,
            default_unigram_score_for_short,
            default_unigram_score_for_long,
        }
    }

    pub fn new_with_default_score(
        system_kana_kanji_dict: KanaKanjiDict,
        system_single_term_dict: KanaKanjiDict,
        user_data: Arc<Mutex<UserData>>,
        system_unigram_lm: Rc<SystemUnigramLM>,
        system_bigram_lm: Rc<SystemBigramLM>,
    ) -> GraphBuilder {
        Self::new(
            system_kana_kanji_dict,
            system_single_term_dict,
            user_data,
            system_unigram_lm,
            system_bigram_lm,
            19_f32,
            20_f32,
        )
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

                let mut seen: HashSet<String> = HashSet::new();

                // 漢字に変換した結果もあれば insert する。
                if let Some(kanjis) = self.system_kana_kanji_dict.find(segmented_yomi) {
                    for kanji in kanjis {
                        let node = WordNode::new(
                            (end_pos - segmented_yomi.len()) as i32,
                            &kanji,
                            segmented_yomi,
                        );
                        vec.push(node);
                        seen.insert(kanji.to_string());
                    }
                }
                for surface in [
                    segmented_yomi,
                    hira2kata(segmented_yomi, ConvOption::default()).as_str(),
                ] {
                    if seen.contains(surface) {
                        continue;
                    }
                    // ひらがなそのものと、カタカナ表現もエントリーとして登録しておく。
                    let node = WordNode::new(
                        (end_pos - segmented_yomi.len()) as i32,
                        surface,
                        segmented_yomi,
                    );
                    vec.push(node);
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
            default_unigram_score_for_long: 20.0_f32,
            default_unigram_score_for_short: 19.0_f32,
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
        let graph_builder = GraphBuilder::new_with_default_score(
            KanaKanjiDict::default(),
            KanaKanjiDictBuilder::default().add("すし", "🍣").build(),
            Arc::new(Mutex::new(UserData::default())),
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
        assert_eq!(
            got_surfaces,
            vec!["すし".to_string(), "スシ".to_string(), "🍣".to_string()]
        );
    }

    // ひらがな、カタカナのエントリーが自動的に入るようにする。
    #[test]
    fn test_default_terms() {
        let graph_builder = GraphBuilder::new_with_default_score(
            KanaKanjiDict::default(),
            KanaKanjiDictBuilder::default().build(),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(SystemUnigramLMBuilder::default().build()),
            Rc::new(SystemBigramLMBuilder::default().build()),
        );
        let yomi = "す";
        let got = graph_builder.construct(
            yomi,
            SegmentationResult::new(BTreeMap::from([(3, vec!["す".to_string()])])),
        );
        let nodes = got.node_list(3).unwrap();
        let got_surfaces: Vec<String> = nodes.iter().map(|f| f.kanji.to_string()).collect();
        assert_eq!(got_surfaces, vec!["す".to_string(), "ス".to_string()]);
    }

    // ひらがな、カタカナがすでにかな漢字辞書から提供されている場合でも、重複させない。
    #[test]
    fn test_default_terms_duplicated() {
        let graph_builder = GraphBuilder::new_with_default_score(
            KanaKanjiDictBuilder::default().add("す", "す/ス").build(),
            KanaKanjiDictBuilder::default().build(),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(SystemUnigramLMBuilder::default().build()),
            Rc::new(SystemBigramLMBuilder::default().build()),
        );
        let yomi = "す";
        let got = graph_builder.construct(
            yomi,
            SegmentationResult::new(BTreeMap::from([(3, vec!["す".to_string()])])),
        );
        let nodes = got.node_list(3).unwrap();
        let got_surfaces: Vec<String> = nodes.iter().map(|f| f.kanji.to_string()).collect();
        assert_eq!(got_surfaces, vec!["す".to_string(), "ス".to_string()]);
    }
}
