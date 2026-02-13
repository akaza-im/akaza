use std::collections::btree_map::BTreeMap;
use std::rc::Rc;

use rustc_hash::FxHashSet;
use std::sync::{Arc, Mutex};

use kelp::{hira2kata, ConvOption};
use log::trace;
use regex::Regex;

use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::SegmentationResult;
use crate::graph::word_node::{WordNode, BOS_TOKEN_KEY, EOS_TOKEN_KEY};
use crate::kana_kanji::base::KanaKanjiDict;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};
use crate::user_side_data::user_data::UserData;

/// surface が数字+接尾辞の場合、LM lookup 用のキーを `<NUM>` 正規化する。
/// `libakaza` は `akaza-data` に依存しないため、同等のロジックをインラインで持つ。
///
/// 裸の数字（suffix なし）はフォールバックしない。全数字カウント集約により
/// `<NUM>/<NUM>` のスコアが極端に高くなり、「に→2」「さん→3」等の退行を起こすため。
fn normalize_surface_for_lm(key: &str) -> Option<String> {
    let slash_pos = key.find('/')?;
    let surface = &key[..slash_pos];
    let digit_end = surface.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digit_end == 0 {
        return None;
    }
    let suffix = &surface[digit_end..];
    if suffix.is_empty() {
        // 裸の数字はフォールバックしない
        None
    } else {
        Some(format!("<NUM>{0}/<NUM>{0}", suffix))
    }
}

pub struct GraphBuilder<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> {
    system_kana_kanji_dict: KD,
    system_single_term_dict: KD,
    user_data: Arc<Mutex<UserData>>,
    system_unigram_lm: Rc<U>,
    system_bigram_lm: Rc<B>,
    number_pattern: Regex,
}

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> GraphBuilder<U, B, KD> {
    pub fn new(
        system_kana_kanji_dict: KD,
        system_single_term_dict: KD,
        user_data: Arc<Mutex<UserData>>,
        system_unigram_lm: Rc<U>,
        system_bigram_lm: Rc<B>,
    ) -> GraphBuilder<U, B, KD> {
        let number_pattern = Regex::new(r#"^[0-9]+"#).unwrap();
        GraphBuilder {
            system_kana_kanji_dict,
            system_single_term_dict,
            user_data,
            system_unigram_lm,
            system_bigram_lm,
            number_pattern,
        }
    }

    pub fn construct(&self, yomi: &str, words_ends_at: &SegmentationResult) -> LatticeGraph<U, B> {
        // このグラフのインデクスは単語の終了位置。
        let mut graph: BTreeMap<i32, Vec<WordNode>> = BTreeMap::new();

        let mut bos = WordNode::create_bos();
        if let Some((word_id, _)) = self.system_unigram_lm.find(BOS_TOKEN_KEY) {
            bos.word_id_and_score = Some((word_id, 0.0)); // score=0: ノードコストは0のまま
        }
        graph.insert(0, vec![bos]);

        let mut eos = WordNode::create_eos(yomi.len() as i32);
        if let Some((word_id, _)) = self.system_unigram_lm.find(EOS_TOKEN_KEY) {
            eos.word_id_and_score = Some((word_id, 0.0));
        }
        graph.insert((yomi.len() + 1) as i32, vec![eos]);

        let mut key_buf = String::new();
        let mut seen: FxHashSet<String> = FxHashSet::default();

        for (end_pos, segmented_yomis) in words_ends_at.iter() {
            for segmented_yomi in segmented_yomis {
                let vec = graph.entry(*end_pos as i32).or_default();

                seen.clear();

                // TODO このへんコピペすぎるので整理必要。
                // システム辞書にある候補を元に候補をリストアップする
                if let Some(kanjis) = self.system_kana_kanji_dict.get(segmented_yomi) {
                    for kanji in kanjis {
                        key_buf.clear();
                        key_buf.push_str(&kanji);
                        key_buf.push('/');
                        key_buf.push_str(segmented_yomi);
                        let word_id_and_score =
                            self.system_unigram_lm.find(&key_buf).or_else(|| {
                                normalize_surface_for_lm(&key_buf)
                                    .and_then(|nk| self.system_unigram_lm.find(&nk))
                            });
                        let node = WordNode::new(
                            (end_pos - segmented_yomi.len()) as i32,
                            &kanji,
                            segmented_yomi,
                            word_id_and_score,
                            false,
                        );
                        trace!("WordIDScore: {:?}", node.word_id_and_score);
                        vec.push(node);
                        seen.insert(kanji.to_string());
                    }
                }
                if let Some(surfaces) = self.user_data.lock().unwrap().dict.get(segmented_yomi) {
                    for surface in surfaces {
                        if seen.contains(surface) {
                            continue;
                        }
                        key_buf.clear();
                        key_buf.push_str(surface);
                        key_buf.push('/');
                        key_buf.push_str(segmented_yomi);
                        let word_id_and_score =
                            self.system_unigram_lm.find(&key_buf).or_else(|| {
                                normalize_surface_for_lm(&key_buf)
                                    .and_then(|nk| self.system_unigram_lm.find(&nk))
                            });
                        let node = WordNode::new(
                            (end_pos - segmented_yomi.len()) as i32,
                            surface,
                            segmented_yomi,
                            word_id_and_score,
                            false,
                        );
                        trace!("WordIDScore: {:?}", node.word_id_and_score);
                        vec.push(node);
                        seen.insert(surface.to_string());
                    }
                }
                // ひらがな候補をリストアップする
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
                        None,
                        true,
                    );
                    vec.push(node);
                }

                // 数字の場合は数字用の動的変換を入れる
                if self.number_pattern.is_match(segmented_yomi) {
                    let node = WordNode::new(
                        (end_pos - segmented_yomi.len()) as i32,
                        "(*(*(NUMBER-KANSUJI",
                        segmented_yomi,
                        None,
                        true,
                    );
                    vec.push(node);
                }

                // 変換範囲が全体になっていれば single term 辞書を利用する。
                if segmented_yomi == yomi {
                    if let Some(surfaces) = self.system_single_term_dict.get(yomi) {
                        for surface in surfaces {
                            key_buf.clear();
                            key_buf.push_str(&surface);
                            key_buf.push('/');
                            key_buf.push_str(segmented_yomi);
                            let word_id_and_score =
                                self.system_unigram_lm.find(&key_buf).or_else(|| {
                                    normalize_surface_for_lm(&key_buf)
                                        .and_then(|nk| self.system_unigram_lm.find(&nk))
                                });
                            let node = WordNode::new(
                                (end_pos - segmented_yomi.len()) as i32,
                                &surface,
                                segmented_yomi,
                                word_id_and_score,
                                false,
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
    use std::collections::HashMap;

    use crate::kana_kanji::hashmap_vec::HashmapVecKanaKanjiDict;
    use crate::lm::system_bigram::MarisaSystemBigramLMBuilder;
    use crate::lm::system_unigram_lm::MarisaSystemUnigramLMBuilder;

    use super::*;

    #[test]
    fn test_single_term() -> anyhow::Result<()> {
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            HashmapVecKanaKanjiDict::new(HashMap::from([(
                "すし".to_string(),
                vec!["🍣".to_string()],
            )])),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(
                MarisaSystemUnigramLMBuilder::default()
                    .set_unique_words(20)
                    .set_total_words(19)
                    .build()?,
            ),
            Rc::new(
                MarisaSystemBigramLMBuilder::default()
                    .set_default_edge_cost(20_f32)
                    .build()?,
            ),
        );
        let yomi = "すし";
        let got = graph_builder.construct(
            yomi,
            &SegmentationResult::new(BTreeMap::from([(6, vec!["すし".to_string()])])),
        );
        let nodes = got.node_list(6).unwrap();
        let got_surfaces: Vec<String> = nodes.iter().map(|f| f.surface.to_string()).collect();
        assert_eq!(
            got_surfaces,
            vec!["すし".to_string(), "スシ".to_string(), "🍣".to_string()]
        );
        Ok(())
    }

    // ひらがな、カタカナのエントリーが自動的に入るようにする。
    #[test]
    fn test_default_terms() -> anyhow::Result<()> {
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(
                MarisaSystemUnigramLMBuilder::default()
                    .set_unique_words(20)
                    .set_total_words(19)
                    .build()?,
            ),
            Rc::new(
                MarisaSystemBigramLMBuilder::default()
                    .set_default_edge_cost(20_f32)
                    .build()?,
            ),
        );
        let yomi = "す";
        let got = graph_builder.construct(
            yomi,
            &SegmentationResult::new(BTreeMap::from([(3, vec!["す".to_string()])])),
        );
        let nodes = got.node_list(3).unwrap();
        let got_surfaces: Vec<String> = nodes.iter().map(|f| f.surface.to_string()).collect();
        assert_eq!(got_surfaces, vec!["す".to_string(), "ス".to_string()]);
        Ok(())
    }

    // ひらがな、カタカナがすでにかな漢字辞書から提供されている場合でも、重複させない。
    #[test]
    fn test_default_terms_duplicated() -> anyhow::Result<()> {
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(HashMap::from([(
                "す".to_string(),
                vec!["す".to_string(), "ス".to_string()],
            )])),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(
                MarisaSystemUnigramLMBuilder::default()
                    .set_unique_words(20)
                    .set_total_words(19)
                    .build()?,
            ),
            Rc::new(
                MarisaSystemBigramLMBuilder::default()
                    .set_default_edge_cost(20_f32)
                    .build()?,
            ),
        );
        let yomi = "す";
        let got = graph_builder.construct(
            yomi,
            &SegmentationResult::new(BTreeMap::from([(3, vec!["す".to_string()])])),
        );
        let nodes = got.node_list(3).unwrap();
        let got_surfaces: Vec<String> = nodes.iter().map(|f| f.surface.to_string()).collect();
        assert_eq!(got_surfaces, vec!["す".to_string(), "ス".to_string()]);
        Ok(())
    }

    #[test]
    fn test_normalize_surface_for_lm() {
        assert_eq!(
            normalize_surface_for_lm("1匹/1ひき"),
            Some("<NUM>匹/<NUM>匹".to_string())
        );
        assert_eq!(
            normalize_surface_for_lm("100円/100えん"),
            Some("<NUM>円/<NUM>円".to_string())
        );
        // 裸の数字はフォールバックしない（スコア集約による退行を防止）
        assert_eq!(normalize_surface_for_lm("1/1"), None);
        assert_eq!(normalize_surface_for_lm("匹/ひき"), None);
        assert_eq!(normalize_surface_for_lm("第1回/だい1かい"), None);
    }
}
