use std::collections::btree_map::BTreeMap;
use std::rc::Rc;

use rustc_hash::FxHashSet;
use std::sync::{Arc, Mutex};

use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::segmenter::SegmentationResult;
use crate::graph::word_node::{WordNode, BOS_TOKEN_KEY, EOS_TOKEN_KEY};
use crate::kana_kanji::base::KanaKanjiDict;
use crate::kansuji::int2kanji;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};
use crate::numeric_counter::{
    counter_surfaces_for, normalize_counter_key_for_lm, normalize_counter_yomi,
    parse_kana_numeric_prefix_before_counter, parse_numeric_prefix_surface, to_fullwidth_digits,
};
use crate::user_side_data::user_data::UserData;
use kelp::{hira2kata, z2h, ConvOption};
use log::trace;

/// surface が数字+接尾辞の場合、LM lookup 用のキーを `<NUM>` 正規化する。
/// `libakaza` は `akaza-data` に依存しないため、同等のロジックをインラインで持つ。
///
/// 裸の数字（suffix なし）はフォールバックしない。全数字カウント集約により
/// `<NUM>/<NUM>` のスコアが極端に高くなり、「に→2」「さん→3」等の退行を起こすため。
///
/// surface 側は漢字接尾辞を保持し、reading 側はかな読みを保持する。
/// - `"90行/90ぎょう"` → `"<NUM>行/<NUM>ぎょう"`
fn normalize_surface_for_lm(key: &str) -> Option<String> {
    if let Some(normalized_counter) = normalize_counter_key_for_lm(key) {
        return Some(normalized_counter);
    }

    let slash_pos = key.find('/')?;
    let surface = &key[..slash_pos];
    let reading = &key[slash_pos + 1..];

    // 助数詞単漢字フォールバック: "年/ねん" → "<NUM>年/<NUM>ねん"
    // 数字なしでも助数詞表面が辞書候補にある場合、<NUM> 正規化版のスコアを参照する。
    if let Some(canonical_yomi) = normalize_counter_yomi(reading) {
        if let Some(surfaces) = counter_surfaces_for(canonical_yomi) {
            if surfaces.contains(&surface) {
                return Some(format!("<NUM>{surface}/<NUM>{canonical_yomi}"));
            }
        }
    }

    let surface_prefix_end = parse_numeric_prefix_surface(surface)
        .map(|p| p.consumed_len)
        .unwrap_or(0);
    if surface_prefix_end == 0 {
        return None;
    }
    let surface_suffix = &surface[surface_prefix_end..];
    if surface_suffix.is_empty() {
        // 裸の数字はフォールバックしない
        None
    } else {
        // reading 側も先頭の数字部分を <NUM> に置換し、かな読みを保持
        let reading_prefix_end = parse_numeric_prefix_surface(reading)
            .map(|p| p.consumed_len)
            .unwrap_or(0);
        if reading_prefix_end == 0 {
            return None;
        }
        let reading_suffix = &reading[reading_prefix_end..];
        Some(format!("<NUM>{surface_suffix}/<NUM>{reading_suffix}"))
    }
}

pub struct GraphBuilder<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> {
    system_kana_kanji_dict: KD,
    system_single_term_dict: KD,
    user_data: Arc<Mutex<UserData>>,
    system_unigram_lm: Rc<U>,
    system_bigram_lm: Rc<B>,
    unknown_bigram_cost: f32,
}

impl<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict> GraphBuilder<U, B, KD> {
    pub fn new(
        system_kana_kanji_dict: KD,
        system_single_term_dict: KD,
        user_data: Arc<Mutex<UserData>>,
        system_unigram_lm: Rc<U>,
        system_bigram_lm: Rc<B>,
    ) -> GraphBuilder<U, B, KD> {
        let unknown_bigram_cost = system_bigram_lm.get_default_edge_cost();
        GraphBuilder {
            system_kana_kanji_dict,
            system_single_term_dict,
            user_data,
            system_unigram_lm,
            system_bigram_lm,
            unknown_bigram_cost,
        }
    }

    pub fn with_unknown_bigram_cost(mut self, cost: f32) -> Self {
        self.unknown_bigram_cost = cost;
        self
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
                let katakana = hira2kata(segmented_yomi, ConvOption::default());
                let half_katakana = z2h(katakana.as_str(), ConvOption::default());
                for surface in [segmented_yomi, katakana.as_str(), half_katakana.as_str()] {
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

                // 数字トークン: 数値表記候補（半角/全角/漢数字）を補完する
                if let Some(numeric) = parse_numeric_prefix_surface(segmented_yomi)
                    .or_else(|| parse_kana_numeric_prefix_before_counter(segmented_yomi))
                {
                    let start_pos = (end_pos - segmented_yomi.len()) as i32;
                    if numeric.consumed_len == segmented_yomi.len() {
                        let fullwidth = to_fullwidth_digits(&numeric.ascii_digits);
                        if !seen.contains(&fullwidth) {
                            vec.push(WordNode::new(
                                start_pos,
                                &fullwidth,
                                segmented_yomi,
                                None,
                                true,
                            ));
                            seen.insert(fullwidth);
                        }
                        let kansuji = int2kanji(numeric.value);
                        if !seen.contains(&kansuji) {
                            vec.push(WordNode::new(
                                start_pos,
                                &kansuji,
                                segmented_yomi,
                                None,
                                true,
                            ));
                            seen.insert(kansuji);
                        }
                    }

                    // 数字+かな複合セグメント（例: "90ぎょう", "５１６しゅうかん", "ぜろひき"）
                    if numeric.consumed_len < segmented_yomi.len() {
                        let kana_part = &segmented_yomi[numeric.consumed_len..];
                        let mut kanjis: Vec<String> = Vec::new();
                        let mut lm_kana_part = kana_part;

                        if let Some(from_dict) = self.system_kana_kanji_dict.get(kana_part) {
                            kanjis.extend(from_dict.iter().cloned());
                        }
                        if let Some(canonical_yomi) = normalize_counter_yomi(kana_part) {
                            lm_kana_part = canonical_yomi;
                            if let Some(counter_surfaces) = counter_surfaces_for(canonical_yomi) {
                                for s in counter_surfaces {
                                    kanjis.push((*s).to_string());
                                }
                            }
                        }

                        let mut seen_kanji: FxHashSet<String> = FxHashSet::default();
                        for kanji in kanjis {
                            if !seen_kanji.insert(kanji.clone()) {
                                continue;
                            }
                            let ascii_surface = format!("{}{}", numeric.ascii_digits, kanji);
                            let fullwidth_surface =
                                format!("{}{}", to_fullwidth_digits(&numeric.ascii_digits), kanji);
                            let kansuji_surface = format!("{}{}", int2kanji(numeric.value), kanji);

                            let reading_for_lm =
                                format!("{}{}", numeric.ascii_digits, lm_kana_part);
                            key_buf.clear();
                            key_buf.push_str(&ascii_surface);
                            key_buf.push('/');
                            key_buf.push_str(&reading_for_lm);
                            let word_id_and_score = {
                                let direct = self.system_unigram_lm.find(&key_buf);
                                let normalized = normalize_surface_for_lm(&key_buf)
                                    .and_then(|nk| self.system_unigram_lm.find(&nk));
                                match (direct, normalized) {
                                    (Some(d), Some(n)) => Some(if d.1 <= n.1 { d } else { n }),
                                    (d @ Some(_), None) => d,
                                    (None, n @ Some(_)) => n,
                                    (None, None) => None,
                                }
                            };

                            for surface in [&ascii_surface, &fullwidth_surface, &kansuji_surface] {
                                if seen.contains(surface) {
                                    continue;
                                }
                                vec.push(WordNode::new(
                                    start_pos,
                                    surface,
                                    segmented_yomi,
                                    word_id_and_score,
                                    false,
                                ));
                                seen.insert(surface.to_string());
                            }
                        }
                    }
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
            unknown_bigram_cost: self.unknown_bigram_cost,
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
            vec![
                "すし".to_string(),
                "スシ".to_string(),
                "ｽｼ".to_string(),
                "🍣".to_string()
            ]
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
        assert_eq!(
            got_surfaces,
            vec!["す".to_string(), "ス".to_string(), "ｽ".to_string()]
        );
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
        assert_eq!(
            got_surfaces,
            vec!["す".to_string(), "ス".to_string(), "ｽ".to_string()]
        );
        Ok(())
    }

    #[test]
    fn test_normalize_surface_for_lm() {
        assert_eq!(
            normalize_surface_for_lm("1匹/1ひき"),
            Some("<NUM>匹/<NUM>ひき".to_string())
        );
        assert_eq!(
            normalize_surface_for_lm("100円/100えん"),
            Some("<NUM>円/<NUM>えん".to_string())
        );
        assert_eq!(
            normalize_surface_for_lm("90行/90ぎょう"),
            Some("<NUM>行/<NUM>ぎょう".to_string())
        );
        // 裸の数字はフォールバックしない（スコア集約による退行を防止）
        assert_eq!(normalize_surface_for_lm("1/1"), None);
        assert_eq!(
            normalize_surface_for_lm("５１６週間/５１６しゅうかん"),
            Some("<NUM>週間/<NUM>しゅうかん".to_string())
        );
        assert_eq!(
            normalize_surface_for_lm("0匹/ぜろひき"),
            Some("<NUM>匹/<NUM>ひき".to_string())
        );
        // 助数詞単漢字フォールバックにより <NUM> 正規化版を返す
        assert_eq!(
            normalize_surface_for_lm("匹/ひき"),
            Some("<NUM>匹/<NUM>ひき".to_string())
        );
        assert_eq!(normalize_surface_for_lm("第1回/だい1かい"), None);
        // 助数詞単漢字フォールバック
        assert_eq!(
            normalize_surface_for_lm("年/ねん"),
            Some("<NUM>年/<NUM>ねん".to_string())
        );
        assert_eq!(
            normalize_surface_for_lm("円/えん"),
            Some("<NUM>円/<NUM>えん".to_string())
        );
        // 「月/つき」は COUNTER_DEFS にないので None
        assert_eq!(normalize_surface_for_lm("月/つき"), None);
        // 「念/ねん」は助数詞 surface ではないので None
        assert_eq!(normalize_surface_for_lm("念/ねん"), None);
    }

    #[test]
    fn test_numeric_counter_variants() -> anyhow::Result<()> {
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

        let yomi = "ぜろひき";
        let got = graph_builder.construct(
            yomi,
            &SegmentationResult::new(BTreeMap::from([(12, vec![yomi.to_string()])])),
        );
        let nodes = got.node_list(12).unwrap();
        let got_surfaces: Vec<String> = nodes.iter().map(|f| f.surface.to_string()).collect();
        assert!(got_surfaces.contains(&"0匹".to_string()));
        assert!(got_surfaces.contains(&"０匹".to_string()));
        assert!(got_surfaces.contains(&"零匹".to_string()));

        let yomi2 = "516しゅうかん";
        let got2 = graph_builder.construct(
            yomi2,
            &SegmentationResult::new(BTreeMap::from([(18, vec![yomi2.to_string()])])),
        );
        let nodes2 = got2.node_list(18).unwrap();
        let got_surfaces2: Vec<String> = nodes2.iter().map(|f| f.surface.to_string()).collect();
        assert!(got_surfaces2.contains(&"516週間".to_string()));
        assert!(got_surfaces2.contains(&"５１６週間".to_string()));
        assert!(got_surfaces2.contains(&"五百十六週間".to_string()));

        let yomi3 = "にせんにじゅうろくしゅうかん";
        let got3 = graph_builder.construct(
            yomi3,
            &SegmentationResult::new(BTreeMap::from([(42, vec![yomi3.to_string()])])),
        );
        let nodes3 = got3.node_list(42).unwrap();
        let got_surfaces3: Vec<String> = nodes3.iter().map(|f| f.surface.to_string()).collect();
        assert!(got_surfaces3.contains(&"2026週間".to_string()));
        assert!(got_surfaces3.contains(&"２０２６週間".to_string()));
        assert!(got_surfaces3.contains(&"二千二十六週間".to_string()));

        let yomi4 = "3ぼん";
        let got4 = graph_builder.construct(
            yomi4,
            &SegmentationResult::new(BTreeMap::from([(7, vec![yomi4.to_string()])])),
        );
        let nodes4 = got4.node_list(7).unwrap();
        let got_surfaces4: Vec<String> = nodes4.iter().map(|f| f.surface.to_string()).collect();
        assert!(got_surfaces4.contains(&"3本".to_string()));
        assert!(got_surfaces4.contains(&"３本".to_string()));
        assert!(got_surfaces4.contains(&"三本".to_string()));

        let yomi5 = "じゅっぷん";
        let got5 = graph_builder.construct(
            yomi5,
            &SegmentationResult::new(BTreeMap::from([(15, vec![yomi5.to_string()])])),
        );
        let nodes5 = got5.node_list(15).unwrap();
        let got_surfaces5: Vec<String> = nodes5.iter().map(|f| f.surface.to_string()).collect();
        assert!(got_surfaces5.contains(&"10分".to_string()));
        assert!(got_surfaces5.contains(&"１０分".to_string()));
        assert!(got_surfaces5.contains(&"十分".to_string()));
        Ok(())
    }

    /// 数値複合語の LM スコア選択で、リテラルと <NUM> 正規化版の両方を検索し、
    /// スコアが良い方（値が小さい方）を採用することを確認する。
    #[test]
    fn test_numeric_compound_picks_better_score() -> anyhow::Result<()> {
        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        unigram_builder.set_unique_words(100);
        unigram_builder.set_total_words(1000);
        // リテラル "1日/1にち" は悪いスコア（高い値）
        unigram_builder.add("1日/1にち", 9.386);
        // 正規化版 "<NUM>日/<NUM>にち" は良いスコア（低い値）
        unigram_builder.add("<NUM>日/<NUM>にち", 2.123);

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(HashMap::from([(
                "にち".to_string(),
                vec!["日".to_string()],
            )])),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(unigram_builder.build()?),
            Rc::new(
                MarisaSystemBigramLMBuilder::default()
                    .set_default_edge_cost(20_f32)
                    .build()?,
            ),
        );

        let yomi = "1にち";
        let end_pos = yomi.len(); // 7 bytes: 1(1) + に(3) + ち(3)
        let got = graph_builder.construct(
            yomi,
            &SegmentationResult::new(BTreeMap::from([(end_pos, vec![yomi.to_string()])])),
        );
        let nodes = got.node_list(end_pos as i32).unwrap();
        let node_1nichi = nodes.iter().find(|n| n.surface == "1日").unwrap();
        // 正規化版の良いスコア (2.123) が採用されるべき
        let (_word_id, score) = node_1nichi.word_id_and_score.unwrap();
        assert!(
            (score - 2.123_f32).abs() < 0.001,
            "Expected normalized score 2.123, got {}",
            score
        );
        Ok(())
    }

    /// リテラルのスコアが正規化版より良い場合は、リテラルが採用されることを確認する。
    #[test]
    fn test_numeric_compound_keeps_better_literal_score() -> anyhow::Result<()> {
        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        unigram_builder.set_unique_words(100);
        unigram_builder.set_total_words(1000);
        // リテラル "1日/1にち" は良いスコア（低い値）
        unigram_builder.add("1日/1にち", 2.0);
        // 正規化版 "<NUM>日/<NUM>にち" は悪いスコア（高い値）
        unigram_builder.add("<NUM>日/<NUM>にち", 9.0);

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(HashMap::from([(
                "にち".to_string(),
                vec!["日".to_string()],
            )])),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(unigram_builder.build()?),
            Rc::new(
                MarisaSystemBigramLMBuilder::default()
                    .set_default_edge_cost(20_f32)
                    .build()?,
            ),
        );

        let yomi = "1にち";
        let end_pos = yomi.len(); // 7 bytes: 1(1) + に(3) + ち(3)
        let got = graph_builder.construct(
            yomi,
            &SegmentationResult::new(BTreeMap::from([(end_pos, vec![yomi.to_string()])])),
        );
        let nodes = got.node_list(end_pos as i32).unwrap();
        let node_1nichi = nodes.iter().find(|n| n.surface == "1日").unwrap();
        // リテラルの良いスコア (2.0) が採用されるべき
        let (_word_id, score) = node_1nichi.word_id_and_score.unwrap();
        assert!(
            (score - 2.0_f32).abs() < 0.001,
            "Expected literal score 2.0, got {}",
            score
        );
        Ok(())
    }

    /// 助数詞単漢字（"年/ねん" 等）が数字プレフィックスなしでも
    /// <NUM> 正規化版の LM スコアにフォールバックすることを確認する。
    #[test]
    fn test_counter_bare_kanji_lm_fallback() -> anyhow::Result<()> {
        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        unigram_builder.set_unique_words(100);
        unigram_builder.set_total_words(1000);
        unigram_builder.add("<NUM>年/<NUM>ねん", 3.5);

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(HashMap::from([(
                "ねん".to_string(),
                vec!["年".to_string()],
            )])),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(unigram_builder.build()?),
            Rc::new(
                MarisaSystemBigramLMBuilder::default()
                    .set_default_edge_cost(20_f32)
                    .build()?,
            ),
        );

        let yomi = "ねん";
        let end_pos = yomi.len(); // 6 bytes
        let got = graph_builder.construct(
            yomi,
            &SegmentationResult::new(BTreeMap::from([(end_pos, vec![yomi.to_string()])])),
        );
        let nodes = got.node_list(end_pos as i32).unwrap();
        let node_nen = nodes.iter().find(|n| n.surface == "年").unwrap();
        let (_word_id, score) = node_nen.word_id_and_score.unwrap();
        assert!(
            (score - 3.5_f32).abs() < 0.001,
            "Expected <NUM> fallback score 3.5, got {}",
            score
        );
        Ok(())
    }
}
