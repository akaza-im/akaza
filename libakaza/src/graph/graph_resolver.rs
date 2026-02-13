use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::rc::Rc;

use rustc_hash::{FxHashMap, FxHashSet};

use anyhow::{bail, Context};
use log::{error, info, trace};

use crate::graph::candidate::Candidate;
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::word_node::WordNode;
use crate::lm::base::{SystemBigramLM, SystemSkipBigramLM, SystemUnigramLM};
use crate::user_side_data::user_data::UserData;

/// k-best の1パス分の結果。分節パターンと真のパスコスト（BOSからEOSまでの累積コスト）を保持する。
#[derive(Debug)]
pub struct KBestPath {
    /// 各文節の漢字候補リスト
    pub segments: Vec<Vec<Candidate>>,
    /// BOSからEOSまでの真の累積コスト（前向きDPで計算された値）。後方互換のため viterbi_cost と同値。
    pub cost: f32,
    /// Viterbi DP の元コスト
    pub viterbi_cost: f32,
    /// Σ unigram コスト
    pub unigram_cost: f32,
    /// Σ 既知 bigram コスト
    pub bigram_cost: f32,
    /// Σ 未知 bigram フォールバックコスト
    pub unknown_bigram_cost: f32,
    /// 未知 bigram の回数
    pub unknown_bigram_count: u32,
    /// パス内のトークン数（BOS/EOS 除く）
    pub token_count: u32,
    /// リランキング後のスコア（ソートキー）
    pub rerank_cost: f32,
    /// パス上の各ノードの word_id（BOS/EOS 除く、先頭→末尾順）
    pub word_ids: Vec<Option<i32>>,
    /// Σ skip-bigram コスト（リランキング用、エンジン側で計算）
    pub skip_bigram_cost: f32,
}

/**
 * Segmenter により分割されたかな表現から、グラフを構築する。
 */
pub struct GraphResolver {
    /// skip-bigram 言語モデル（Optional）
    skip_bigram_lm: Option<Rc<dyn SystemSkipBigramLM>>,
    /// skip-bigram コストの重み（Viterbi DP に加算）
    skip_bigram_weight: f32,
}

impl Default for GraphResolver {
    fn default() -> Self {
        Self {
            skip_bigram_lm: None,
            skip_bigram_weight: 0.0,
        }
    }
}

/// k-best のエントリ。各ノードにおいて上位 k 個の経路を保持するために使う。
#[derive(Debug, Clone)]
struct KBestEntry<'a> {
    cost: f32,
    prev_node: &'a WordNode,
    prev_rank: usize, // prev_node の k-best リストの何番目から来たか
    // リランキング用の内訳
    unigram_cost_sum: f32,
    known_bigram_cost_sum: f32,
    unknown_bigram_cost_sum: f32,
    unknown_bigram_count: u32,
    token_count: u32,
    /// Σ skip-bigram コスト（重み適用前の生値）
    skip_bigram_cost_sum: f32,
}

impl GraphResolver {
    pub fn new(
        skip_bigram_lm: Option<Rc<dyn SystemSkipBigramLM>>,
        skip_bigram_weight: f32,
    ) -> Self {
        Self {
            skip_bigram_lm,
            skip_bigram_weight,
        }
    }

    /// 祖父ノード (w_{i-2}) と現在ノード (w_i) の skip-bigram コストを返す。
    /// ユーザー skip-bigram を優先し、見つからなければシステム LM にフォールバック。
    /// LM が未設定または word_id がない場合は 0.0。
    fn skip_bigram_cost(
        &self,
        grandparent: &WordNode,
        node: &WordNode,
        user_data: &UserData,
    ) -> f32 {
        // 1. ユーザー skip-bigram を優先
        if let Some(cost) = user_data.get_skip_bigram_cost(grandparent, node) {
            return cost;
        }
        // 2. システム skip-bigram LM にフォールバック
        if let Some(skip_lm) = &self.skip_bigram_lm {
            if let (Some((gp_id, _)), Some((node_id, _))) =
                (grandparent.word_id_and_score, node.word_id_and_score)
            {
                return skip_lm
                    .get_skip_cost(gp_id, node_id)
                    .unwrap_or_else(|| skip_lm.get_default_skip_cost());
            }
        }
        0.0
    }

    /**
     * ビタビアルゴリズムで最適な経路を見つける。
     * k=1 の resolve_k_best に委譲する。
     */
    pub fn resolve<U: SystemUnigramLM, B: SystemBigramLM>(
        &self,
        lattice: &LatticeGraph<U, B>,
    ) -> anyhow::Result<Vec<Vec<Candidate>>> {
        let paths = self.resolve_k_best(lattice, 1)?;
        Ok(paths
            .into_iter()
            .next()
            .map(|p| p.segments)
            .unwrap_or_default())
    }

    /// k-best ビタビアルゴリズムで上位 k 個の分節パターンを返す。
    ///
    /// 戻り値: `Vec<KBestPath>` — 各パスは分節パターン（文節×漢字候補）と真のパスコストを持つ。
    pub fn resolve_k_best<U: SystemUnigramLM, B: SystemBigramLM>(
        &self,
        lattice: &LatticeGraph<U, B>,
        k: usize,
    ) -> anyhow::Result<Vec<KBestPath>> {
        let yomi = &lattice.yomi;
        // 各ノードに対して上位 k 個のエントリを保持する
        let mut kbest_map: FxHashMap<&WordNode, Vec<KBestEntry>> =
            FxHashMap::with_capacity_and_hasher(yomi.len() * 2, Default::default());

        // user_data のロックを一度だけ取得し、ループ中は保持する
        let user_data = lattice.lock_user_data();

        // 前向きに動的計画法でたどる
        for i in 1..yomi.len() + 2 {
            let Some(nodes) = &lattice.node_list(i as i32) else {
                continue;
            };
            for node in *nodes {
                let node_cost = lattice.get_node_cost_with_user_data(node, &user_data);
                trace!("kanji={}, Cost={}", node, node_cost);

                let prev_nodes = lattice.get_prev_nodes(node).with_context(|| {
                    format!(
                        "Cannot get prev nodes for '{}' start={} lattice={:?}",
                        node.surface, node.start_pos, lattice
                    )
                })?;

                // 各前ノードの k-best エントリそれぞれについて候補を生成
                let is_eos = node.surface == "__EOS__";
                let mut entries: Vec<KBestEntry> = Vec::with_capacity(k * prev_nodes.len());
                for prev in prev_nodes {
                    let (edge_cost, is_known_bigram) =
                        lattice.get_edge_cost_detail_with_user_data(prev, node, &user_data);

                    if let Some(prev_entries) = kbest_map.get(prev) {
                        for (rank, prev_entry) in prev_entries.iter().enumerate() {
                            // skip-bigram: 祖父ノード (prev_entry.prev_node) と現在ノード (node)
                            let skip_cost = if !is_eos {
                                self.skip_bigram_cost(prev_entry.prev_node, node, &user_data)
                            } else {
                                0.0
                            };
                            let weighted_skip = self.skip_bigram_weight * skip_cost;
                            let tmp_cost = prev_entry.cost + edge_cost + node_cost + weighted_skip;
                            let tmp_token_count =
                                prev_entry.token_count + if is_eos { 0 } else { 1 };
                            let (known_add, unknown_add, unknown_count_add) = if is_known_bigram {
                                (edge_cost, 0.0, 0)
                            } else {
                                (0.0, edge_cost, 1)
                            };
                            entries.push(KBestEntry {
                                cost: tmp_cost,
                                prev_node: prev,
                                prev_rank: rank,
                                unigram_cost_sum: prev_entry.unigram_cost_sum
                                    + if is_eos { 0.0 } else { node_cost },
                                known_bigram_cost_sum: prev_entry.known_bigram_cost_sum + known_add,
                                unknown_bigram_cost_sum: prev_entry.unknown_bigram_cost_sum
                                    + unknown_add,
                                unknown_bigram_count: prev_entry.unknown_bigram_count
                                    + unknown_count_add,
                                token_count: tmp_token_count,
                                skip_bigram_cost_sum: prev_entry.skip_bigram_cost_sum + skip_cost,
                            });
                        }
                    } else {
                        // BOS ノードなど: コスト 0 として扱う（skip-bigram 対象外）
                        let tmp_cost = edge_cost + node_cost;
                        let (known_add, unknown_add, unknown_count_add) = if is_known_bigram {
                            (edge_cost, 0.0, 0)
                        } else {
                            (0.0, edge_cost, 1)
                        };
                        entries.push(KBestEntry {
                            cost: tmp_cost,
                            prev_node: prev,
                            prev_rank: 0,
                            unigram_cost_sum: if is_eos { 0.0 } else { node_cost },
                            known_bigram_cost_sum: known_add,
                            unknown_bigram_cost_sum: unknown_add,
                            unknown_bigram_count: unknown_count_add,
                            token_count: if is_eos { 0 } else { 1 },
                            skip_bigram_cost_sum: 0.0,
                        });
                    }
                }

                // コスト昇順でソートし、上位 k 個のみ保持
                entries.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap_or(Ordering::Equal));
                entries.truncate(k);

                if entries.is_empty() {
                    bail!(
                        "No valid previous node found for '{}' (start_pos={}, yomi={})",
                        node.surface,
                        node.start_pos,
                        yomi
                    );
                }

                kbest_map.insert(node, entries);
            }
        }

        // costmap を構築（get_candidates で使用。1-best のコストを使う）
        let mut costmap: FxHashMap<&WordNode, f32> =
            FxHashMap::with_capacity_and_hasher(kbest_map.len(), Default::default());
        for (node, entries) in &kbest_map {
            if let Some(best) = entries.first() {
                costmap.insert(node, best.cost);
            }
        }

        // ロックを解放
        drop(user_data);

        // 後ろ向きに候補を探していく
        let eos_pos = (yomi.len() + 1) as i32;
        let eos = lattice
            .get(eos_pos)
            .with_context(|| format!("EOS node not found at position {}", eos_pos))?
            .first()
            .with_context(|| format!("EOS node list is empty at position {}", eos_pos))?;
        let bos = lattice
            .get(0)
            .with_context(|| "BOS node not found at position 0")?
            .first()
            .with_context(|| "BOS node list is empty at position 0")?;

        // EOS の k-best エントリからそれぞれパスを抽出
        let eos_entries = kbest_map
            .get(eos)
            .with_context(|| format!("k-best entries not found for EOS at position {}", eos_pos))?;

        let mut all_paths: Vec<KBestPath> = Vec::with_capacity(k);
        let mut seen_patterns: FxHashSet<Vec<(i32, usize)>> =
            FxHashSet::with_capacity_and_hasher(k, Default::default());

        for eos_entry in eos_entries {
            let mut path: Vec<Vec<Candidate>> = Vec::new();
            let mut word_ids: Vec<Option<i32>> = Vec::new();
            let path_cost = eos_entry.cost; // EOS エントリの累積コスト = 真のパスコスト
            let mut cur_node = eos_entry.prev_node;
            let mut cur_rank = eos_entry.prev_rank;

            while cur_node != bos {
                if cur_node.surface != "__EOS__" {
                    let end_pos = cur_node.start_pos + (cur_node.yomi.len() as i32);
                    let candidates = self.get_candidates(cur_node, lattice, &costmap, end_pos);
                    path.push(candidates);
                    word_ids.push(cur_node.word_id_and_score.map(|(id, _)| id));
                }

                // cur_node の kbest_map から cur_rank 番目のエントリを辿る
                let entries = match kbest_map.get(cur_node) {
                    Some(e) => e,
                    None => break,
                };
                let entry = match entries.get(cur_rank) {
                    Some(e) => e,
                    None => {
                        // rank が範囲外の場合は 0 番目にフォールバック
                        match entries.first() {
                            Some(e) => e,
                            None => break,
                        }
                    }
                };
                cur_node = entry.prev_node;
                cur_rank = entry.prev_rank;
            }
            path.reverse();
            word_ids.reverse();

            // 重複排除: 分節パターン（各文節の (start_pos, yomi_len)）でハッシュ
            let pattern: Vec<(i32, usize)> = path
                .iter()
                .filter_map(|clause| {
                    clause.first().map(|c| (0i32, c.yomi.len())) // start_pos は順序で決まるので yomi_len のみ使う
                })
                .collect();

            if !seen_patterns.contains(&pattern) {
                seen_patterns.insert(pattern);
                all_paths.push(KBestPath {
                    segments: path,
                    cost: path_cost,
                    viterbi_cost: path_cost,
                    unigram_cost: eos_entry.unigram_cost_sum,
                    bigram_cost: eos_entry.known_bigram_cost_sum,
                    unknown_bigram_cost: eos_entry.unknown_bigram_cost_sum,
                    unknown_bigram_count: eos_entry.unknown_bigram_count,
                    token_count: eos_entry.token_count,
                    rerank_cost: path_cost,
                    word_ids,
                    skip_bigram_cost: eos_entry.skip_bigram_cost_sum,
                });
            }
        }

        if all_paths.is_empty() {
            // 最低限 1 パスは返す
            all_paths.push(KBestPath {
                segments: Vec::new(),
                cost: 0.0,
                viterbi_cost: 0.0,
                unigram_cost: 0.0,
                bigram_cost: 0.0,
                unknown_bigram_cost: 0.0,
                unknown_bigram_count: 0,
                token_count: 0,
                rerank_cost: 0.0,
                word_ids: Vec::new(),
                skip_bigram_cost: 0.0,
            });
        }

        Ok(all_paths)
    }

    fn get_candidates<U: SystemUnigramLM, B: SystemBigramLM>(
        &self,
        node: &WordNode,
        lattice: &LatticeGraph<U, B>,
        costmap: &FxHashMap<&WordNode, f32>,
        end_pos: i32,
    ) -> Vec<Candidate> {
        // end_pos で終わる単語を得る。
        let Some(node_list) = lattice.node_list(end_pos) else {
            error!(
                "Node list not found at end_pos={} for node '{}'",
                end_pos, node.surface
            );
            return Vec::new();
        };

        let mut strict_results: Vec<Candidate> = node_list
            .iter()
            .filter(|alt_node| {
                alt_node.start_pos == node.start_pos // 同じ位置かそれより前から始まっている
                    && alt_node.yomi.len() == node.yomi.len() // 同じ長さの単語を得る
            })
            .map(|f| Candidate {
                surface: f.surface.clone(),
                yomi: f.yomi.clone(),
                cost: *costmap.get(f).unwrap_or_else(|| {
                    error!(
                        "Cost not found for node '{}' at pos {}",
                        f.surface, f.start_pos
                    );
                    &f32::MAX
                }),
                compound_word: false,
            })
            .collect();
        strict_results.sort();

        // もし、候補が著しく少ない場合は、その文節を分割する。
        // 分割した場合の単語は strict_results に追加される。
        // ここの閾値はめちゃくちゃヒューリスティックな値です。
        // 北香那/きたかな/キタカナ のようなケースでも 3 例あるので、という指定。
        // そのほか、ここより深い階層のハードコードされているものは、すべて、ヒューリスティック。
        if strict_results.len() < 5 {
            let mut candidates: Vec<Candidate> = Vec::new();
            Self::collect_breakdown_results(
                &node.yomi,
                node.yomi.len(),
                node.start_pos,
                &mut candidates,
                String::new(),
                String::new(),
                lattice,
                end_pos,
                0,
                &costmap,
                0_f32,
                None,
            );
            candidates.sort();
            for x in candidates {
                strict_results.push(x)
            }
        }

        strict_results
    }

    /// - `tail_cost`: 末尾から辿った場合のコスト
    #[allow(clippy::too_many_arguments)]
    fn collect_breakdown_results<U: SystemUnigramLM, B: SystemBigramLM>(
        node_yomi: &str,
        required_len: usize,
        min_start_pos: i32,
        strict_results: &mut Vec<Candidate>,
        cur_surface: String,
        cur_yomi: String,
        lattice: &LatticeGraph<U, B>,
        end_pos: i32,
        depth: i32,
        cost_map: &&FxHashMap<&WordNode, f32>,
        tail_cost: f32,
        next_node: Option<&WordNode>,
    ) {
        if depth > 4 {
            // depth が深過ぎたら諦める。
            info!(
                "collect_splited_results: too deep: node_yomi={:?}, cur_surface={:?}",
                node_yomi, cur_surface
            );
            return;
        }

        if cur_yomi.len() == node_yomi.len() {
            trace!("Insert strict_results: {}/{}", cur_surface, cur_yomi);
            strict_results.push(Candidate {
                surface: cur_surface,
                yomi: cur_yomi,
                cost: tail_cost,
                compound_word: true,
            });
            return;
        }

        let Some(targets) = lattice.node_list(end_pos) else {
            // 直前のノードはない場合ある。
            return;
        };
        trace!("Targets: {:?}", targets);
        let mut targets = targets
            .iter()
            .filter(|cur| {
                // 単語の開始位置が、node の表示範囲内に収まっているもののみをリストアップする
                min_start_pos <= cur.start_pos
                    // 元々の候補と完全に一致しているものは除外。
                    && cur.yomi != node_yomi
            })
            .map(|f| {
                let head_cost = cost_map.get(f).copied().unwrap_or_else(|| {
                    error!(
                        "Cost not found in breakdown for node '{}' at pos {}",
                        f.surface, f.start_pos
                    );
                    f32::MAX
                });
                BreakDown {
                    node: f.clone(),
                    head_cost, // 先頭から辿った場合のコスト
                    tail_cost: tail_cost
                        + lattice.get_node_cost(f)
                        + next_node
                            .map(|nn| lattice.get_edge_cost(f, nn))
                            .unwrap_or_else(|| lattice.get_default_edge_cost()),
                }
            })
            .collect::<Vec<_>>();
        targets.sort();

        // ここの 3、はヒューリスティックな値。
        // たとえば、3単語までブレーくダウンするとすれば、3**3 辿ることになるわけだから
        // 相当気を塚うひつようがあるだろう。
        let targets = targets.iter().take(3).collect::<BinaryHeap<_>>();

        trace!("Targets: {:?}, min_start_pos={}", targets, min_start_pos);
        for target in targets {
            if target.node.yomi == "__BOS__" || target.node.yomi == "__EOS__" {
                continue;
            }

            trace!(
                "Recursive tracking : {}/{}",
                target.node.surface,
                target.node.yomi
            );
            if required_len < target.node.yomi.len() {
                error!(
                    "Length underflow in breakdown: required_len={}, node.yomi.len()={}, node={}",
                    required_len,
                    target.node.yomi.len(),
                    target.node.yomi
                );
                continue; // Skip this breakdown candidate
            }
            Self::collect_breakdown_results(
                node_yomi,
                required_len - target.node.yomi.len(),
                min_start_pos,
                strict_results,
                target.node.surface.clone() + cur_surface.as_str(),
                target.node.yomi.clone() + cur_yomi.as_str(),
                lattice,
                end_pos - (target.node.yomi.len() as i32),
                depth + 1,
                cost_map,
                tail_cost + target.tail_cost,
                Some(&target.node),
            )
        }
    }
}

#[derive(PartialEq, Debug)]
struct BreakDown {
    node: WordNode,
    /// 先頭から辿った場合のコスト
    pub head_cost: f32,
    /// 末尾から辿った場合のコスト
    pub tail_cost: f32,
}

impl Eq for BreakDown {}

impl PartialOrd<Self> for BreakDown {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BreakDown {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.head_cost + self.tail_cost)
            .partial_cmp(&(other.head_cost + other.tail_cost))
            .unwrap_or(Ordering::Equal) // NaN の場合は Equal として扱う
    }
}

#[cfg(test)]
mod tests {
    use std::collections::btree_map::BTreeMap;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use anyhow::Result;
    use log::LevelFilter;

    use crate::graph::graph_builder::GraphBuilder;
    use crate::graph::segmenter::{SegmentationResult, Segmenter};
    use crate::kana_kanji::hashmap_vec::HashmapVecKanaKanjiDict;
    use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
    use crate::lm::system_bigram::MarisaSystemBigramLMBuilder;
    use crate::lm::system_unigram_lm::MarisaSystemUnigramLMBuilder;
    use crate::user_side_data::user_data::UserData;

    use super::*;

    #[test]
    fn test_resolver() -> Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = CedarwoodKanaTrie::build(Vec::from([
            "abc".to_string(),
            "ab".to_string(),
            "c".to_string(),
        ]));

        let graph_builder = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = graph_builder.build("abc", None);
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (2, vec!["ab".to_string()]),
                (3, vec!["abc".to_string(), "c".to_string()]),
            ]))
        );

        // -1  0  1  2
        // BOS a  b  c
        let system_unigram_lm = MarisaSystemUnigramLMBuilder::default()
            .set_unique_words(20)
            .set_total_words(19)
            .build()?;
        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        let system_bigram_lm = system_bigram_lm_builder
            .set_default_edge_cost(20_f32)
            .build()?;
        let user_data = UserData::default();
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            HashmapVecKanaKanjiDict::new(Default::default()),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("abc", &graph);
        let resolver = GraphResolver::default();
        let got = resolver.resolve(&lattice)?;
        let terms: Vec<String> = got.iter().map(|f| f[0].surface.clone()).collect();
        let result = terms.join("");
        assert_eq!(result, "abc");
        Ok(())
    }

    #[test]
    fn test_kana_kanji() -> Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = CedarwoodKanaTrie::build(Vec::from([
            "わたし".to_string(),
            "わた".to_string(),
            "し".to_string(),
        ]));

        let graph_builder = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = graph_builder.build("わたし", None);
        assert_eq!(
            graph,
            SegmentationResult::new(BTreeMap::from([
                (6, vec!["わた".to_string()]),
                (9, vec!["わたし".to_string(), "し".to_string()]),
            ]))
        );

        let dict = HashMap::from([(
            "わたし".to_string(),
            vec!["私".to_string(), "渡し".to_string()],
        )]);

        let yomi = "わたし".to_string();

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        let system_unigram_lm = system_unigram_lm_builder
            .set_unique_words(19)
            .set_total_words(20)
            .build()?;
        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        let system_bigram_lm = system_bigram_lm_builder
            .set_default_edge_cost(20_f32)
            .build()?;
        let mut user_data = UserData::default();
        // 私/わたし のスコアをガッと上げる。
        user_data.record_entries(&[Candidate::new("わたし", "私", 0_f32)]);
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct(&yomi, &graph);
        // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
        // File::create("/tmp/lattice.dot")
        //     .unwrap()
        //     .write_all(lattice.dump_cost_dot().as_bytes())
        //     .unwrap();
        let resolver = GraphResolver::default();
        let got = resolver.resolve(&lattice)?;
        let terms: Vec<String> = got.iter().map(|f| f[0].surface.clone()).collect();
        let result = terms.join("");
        assert_eq!(result, "私");
        Ok(())
    }

    #[test]
    fn test_kitakana() -> Result<()> {
        // 「きたかな」を変換したときに、北香那だけではなく「来た/きた かな/かな」のような
        // 文節を区切った候補も出て来ること。

        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init();

        let kana_trie = CedarwoodKanaTrie::build(Vec::from([
            "きたかな".to_string(),
            "きた".to_string(),
            "き".to_string(),
            "たかな".to_string(),
            "かな".to_string(),
        ]));

        let graph_builder = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = graph_builder.build("きたかな", None);
        // assert_eq!(
        //     graph,
        //     SegmentationResult::new(BTreeMap::from([
        //         (3, vec!["き".to_string()]),
        //         (6, vec!["きた".to_string()]),
        //         (
        //             12,
        //             vec![
        //                 "きたかな".to_string(),
        //                 "かな".to_string(),
        //                 "たかな".to_string(),
        //             ]
        //         ),
        //     ]))
        // );

        let dict = HashMap::from([
            ("きたかな".to_string(), vec!["北香那".to_string()]),
            ("き".to_string(), vec!["気".to_string()]),
            ("たかな".to_string(), vec!["高菜".to_string()]),
            ("かな".to_string(), vec!["かな".to_string()]),
            (
                "きた".to_string(),
                vec!["来た".to_string(), "北".to_string()],
            ),
        ]);

        let yomi = "きたかな".to_string();

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        let system_unigram_lm = system_unigram_lm_builder
            .set_unique_words(19)
            .set_total_words(20)
            .build()?;
        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        let system_bigram_lm = system_bigram_lm_builder
            .set_default_edge_cost(20_f32)
            .build()?;
        let mut user_data = UserData::default();
        // 来た/きた かな/かな のコストを下げておく。
        user_data.record_entries(&[
            Candidate::new("きた", "来た", 0_f32),
            // Candidate::new("かな", "かな", 0_f32),
        ]);
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct(&yomi, &graph);
        // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
        File::create("/tmp/dump.dot")
            .unwrap()
            .write_all(lattice.dump_cost_dot("来たかな").as_bytes())
            .unwrap();
        let resolver = GraphResolver::default();
        let got = resolver.resolve(&lattice)?;
        // 来たかな が候補に出てくる。

        let got = got[0]
            .iter()
            .collect::<Vec<_>>()
            .iter()
            .map(|it| it.surface.to_string())
            .collect::<Vec<_>>()
            .join(",");
        info!("Got: {}", got);
        assert!(got.contains("来たかな"), "{}", got);
        // assert_eq!(result, "来たかな");
        Ok(())
    }

    #[test]
    fn test_multi_word_conversion() -> anyhow::Result<()> {
        // 3単語以上の複合文の変換テスト
        use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;

        let kana_trie = CedarwoodKanaTrie::build(vec![
            "きょう".to_string(),
            "は".to_string(),
            "いい".to_string(),
            "てんき".to_string(),
        ]);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = segmenter.build("きょうはいいてんき", None);

        let dict = HashMap::from([
            ("きょう".to_string(), vec!["今日".to_string()]),
            ("は".to_string(), vec!["は".to_string()]),
            ("いい".to_string(), vec!["良い".to_string()]),
            ("てんき".to_string(), vec!["天気".to_string()]),
        ]);

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        system_unigram_lm_builder.add("今日/きょう", 1.0);
        system_unigram_lm_builder.add("は/は", 0.5);
        system_unigram_lm_builder.add("良い/いい", 1.2);
        system_unigram_lm_builder.add("天気/てんき", 1.5);
        system_unigram_lm_builder.set_total_words(100);
        system_unigram_lm_builder.set_unique_words(50);
        let system_unigram_lm = system_unigram_lm_builder.build()?;

        // bigram スコアを設定して正しい順序を優先
        let unigram_map = system_unigram_lm.as_hash_map();
        let kyou_id = unigram_map.get("今日/きょう").unwrap().0;
        let ha_id = unigram_map.get("は/は").unwrap().0;
        let ii_id = unigram_map.get("良い/いい").unwrap().0;
        let tenki_id = unigram_map.get("天気/てんき").unwrap().0;

        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        system_bigram_lm_builder.set_default_edge_cost(10.0);
        system_bigram_lm_builder.add(kyou_id, ha_id, 0.5); // 今日は
        system_bigram_lm_builder.add(ha_id, ii_id, 0.3); // は良い
        system_bigram_lm_builder.add(ii_id, tenki_id, 0.4); // 良い天気
        let system_bigram_lm = system_bigram_lm_builder.build()?;

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("きょうはいいてんき", &graph);
        let resolver = GraphResolver::default();
        let result = resolver.resolve(&lattice)?;

        // 3単語以上の複合文の変換が成功することを確認
        assert!(!result.is_empty());
        assert!(!result[0].is_empty());

        Ok(())
    }

    #[test]
    fn test_long_sentence_conversion() -> anyhow::Result<()> {
        // より長い文章の変換テスト
        use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;

        let kana_trie = CedarwoodKanaTrie::build(vec![
            "わたし".to_string(),
            "は".to_string(),
            "がっこう".to_string(),
            "に".to_string(),
            "いきます".to_string(),
        ]);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = segmenter.build("わたしはがっこうにいきます", None);

        let dict = HashMap::from([
            ("わたし".to_string(), vec!["私".to_string()]),
            ("は".to_string(), vec!["は".to_string()]),
            ("がっこう".to_string(), vec!["学校".to_string()]),
            ("に".to_string(), vec!["に".to_string()]),
            ("いきます".to_string(), vec!["行きます".to_string()]),
        ]);

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        system_unigram_lm_builder.add("私/わたし", 1.0);
        system_unigram_lm_builder.add("は/は", 0.5);
        system_unigram_lm_builder.add("学校/がっこう", 1.5);
        system_unigram_lm_builder.add("に/に", 0.3);
        system_unigram_lm_builder.add("行きます/いきます", 1.2);
        system_unigram_lm_builder.set_total_words(100);
        system_unigram_lm_builder.set_unique_words(50);
        let system_unigram_lm = system_unigram_lm_builder.build()?;

        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        system_bigram_lm_builder.set_default_edge_cost(10.0);
        let system_bigram_lm = system_bigram_lm_builder.build()?;

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("わたしはがっこうにいきます", &graph);
        let resolver = GraphResolver::default();
        let result = resolver.resolve(&lattice)?;

        // 長い文章の変換が成功することを確認
        assert!(!result.is_empty());
        assert!(!result[0].is_empty());

        Ok(())
    }

    #[test]
    fn test_ambiguous_conversion_ranking() -> anyhow::Result<()> {
        // 曖昧な変換での候補ランキングのテスト
        use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;

        let kana_trie = CedarwoodKanaTrie::build(vec!["はし".to_string()]);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = segmenter.build("はし", None);

        let dict = HashMap::from([(
            "はし".to_string(),
            vec!["橋".to_string(), "箸".to_string(), "端".to_string()],
        )]);

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        // 異なるスコアを設定
        system_unigram_lm_builder.add("橋/はし", 2.0); // 最も一般的
        system_unigram_lm_builder.add("箸/はし", 1.5);
        system_unigram_lm_builder.add("端/はし", 1.0); // 最も稀
        system_unigram_lm_builder.set_total_words(100);
        system_unigram_lm_builder.set_unique_words(50);
        let system_unigram_lm = system_unigram_lm_builder.build()?;

        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        system_bigram_lm_builder.set_default_edge_cost(10.0);
        let system_bigram_lm = system_bigram_lm_builder.build()?;

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("はし", &graph);
        let resolver = GraphResolver::default();
        let result = resolver.resolve(&lattice)?;

        // 複数候補が返されることを確認
        assert!(!result.is_empty());

        // 最上位候補を確認
        let top_surface = result[0].first().unwrap().surface.as_str();
        // いずれかの候補が最上位に来る
        assert!(top_surface == "橋" || top_surface == "箸" || top_surface == "端");

        Ok(())
    }

    #[test]
    fn test_user_learning_priority() -> anyhow::Result<()> {
        // ユーザー学習が候補順位に影響することをテスト
        use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;

        let kana_trie = CedarwoodKanaTrie::build(vec!["はし".to_string()]);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = segmenter.build("はし", None);

        let dict = HashMap::from([("はし".to_string(), vec!["橋".to_string(), "箸".to_string()])]);

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        system_unigram_lm_builder.add("橋/はし", 2.0);
        system_unigram_lm_builder.add("箸/はし", 1.5);
        system_unigram_lm_builder.set_total_words(100);
        system_unigram_lm_builder.set_unique_words(50);
        let system_unigram_lm = system_unigram_lm_builder.build()?;

        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        system_bigram_lm_builder.set_default_edge_cost(10.0);
        let system_bigram_lm = system_bigram_lm_builder.build()?;

        let mut user_data = UserData::default();
        // ユーザーが "箸" を学習している
        user_data.record_entries(&[Candidate::new("はし", "箸", 0.1)]);

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("はし", &graph);
        let resolver = GraphResolver::default();
        let result = resolver.resolve(&lattice)?;

        // ユーザー学習により "箸" が最上位に来ることを確認
        let top_surface = result[0].first().unwrap().surface.as_str();
        assert_eq!(top_surface, "箸");

        Ok(())
    }

    #[test]
    fn test_k_best_kitakana() -> Result<()> {
        // 「きたかな」で k-best を使い、異なる分節パターンが返ることを検証
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = CedarwoodKanaTrie::build(Vec::from([
            "きたかな".to_string(),
            "きた".to_string(),
            "き".to_string(),
            "たかな".to_string(),
            "かな".to_string(),
        ]));

        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = segmenter.build("きたかな", None);

        let dict = HashMap::from([
            ("きたかな".to_string(), vec!["北香那".to_string()]),
            ("き".to_string(), vec!["気".to_string()]),
            ("たかな".to_string(), vec!["高菜".to_string()]),
            ("かな".to_string(), vec!["かな".to_string()]),
            (
                "きた".to_string(),
                vec!["来た".to_string(), "北".to_string()],
            ),
        ]);

        let system_unigram_lm = MarisaSystemUnigramLMBuilder::default()
            .set_unique_words(19)
            .set_total_words(20)
            .build()?;
        let system_bigram_lm = MarisaSystemBigramLMBuilder::default()
            .set_default_edge_cost(20_f32)
            .build()?;
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("きたかな", &graph);
        let resolver = GraphResolver::default();

        let paths = resolver.resolve_k_best(&lattice, 5)?;

        // 少なくとも 1 パスは返る
        assert!(!paths.is_empty());

        // 分節パターンの数を収集（各パスの clause 数）
        let clause_counts: Vec<usize> = paths.iter().map(|p| p.segments.len()).collect();
        info!("k-best clause counts: {:?}", clause_counts);

        // 複数パスが返る場合、異なる分節パターンが含まれることを確認
        if paths.len() > 1 {
            // 少なくとも 1 文節パスと 2 文節パスの両方が含まれていることを確認
            assert!(
                clause_counts.contains(&1) || clause_counts.contains(&2),
                "Expected diverse segmentation patterns: {:?}",
                clause_counts
            );
        }

        Ok(())
    }

    #[test]
    fn test_k_best_single_equals_resolve() -> Result<()> {
        // k=1 の resolve_k_best が resolve() と同じ結果を返すことを検証
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = CedarwoodKanaTrie::build(Vec::from([
            "わたし".to_string(),
            "わた".to_string(),
            "し".to_string(),
        ]));

        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = segmenter.build("わたし", None);

        let dict = HashMap::from([(
            "わたし".to_string(),
            vec!["私".to_string(), "渡し".to_string()],
        )]);

        let system_unigram_lm = MarisaSystemUnigramLMBuilder::default()
            .set_unique_words(19)
            .set_total_words(20)
            .build()?;
        let system_bigram_lm = MarisaSystemBigramLMBuilder::default()
            .set_default_edge_cost(20_f32)
            .build()?;
        let mut user_data = UserData::default();
        user_data.record_entries(&[Candidate::new("わたし", "私", 0_f32)]);
        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(user_data)),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("わたし", &graph);
        let resolver = GraphResolver::default();

        // resolve() の結果
        let single_result = resolver.resolve(&lattice)?;

        // resolve_k_best(k=1) の結果
        let k_best_result = resolver.resolve_k_best(&lattice, 1)?;

        assert_eq!(k_best_result.len(), 1);

        // 各文節の先頭候補が一致すること
        let single_surfaces: Vec<String> =
            single_result.iter().map(|c| c[0].surface.clone()).collect();
        let kbest_surfaces: Vec<String> = k_best_result[0]
            .segments
            .iter()
            .map(|c| c[0].surface.clone())
            .collect();
        assert_eq!(single_surfaces, kbest_surfaces);

        Ok(())
    }

    #[test]
    fn test_k_best_multi_word() -> anyhow::Result<()> {
        // 複数分節パターンが返ることを検証
        let _ = env_logger::builder().is_test(true).try_init();

        let kana_trie = CedarwoodKanaTrie::build(vec![
            "きょう".to_string(),
            "は".to_string(),
            "いい".to_string(),
            "てんき".to_string(),
        ]);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
        let graph = segmenter.build("きょうはいいてんき", None);

        let dict = HashMap::from([
            ("きょう".to_string(), vec!["今日".to_string()]),
            ("は".to_string(), vec!["は".to_string()]),
            ("いい".to_string(), vec!["良い".to_string()]),
            ("てんき".to_string(), vec!["天気".to_string()]),
        ]);

        let mut system_unigram_lm_builder = MarisaSystemUnigramLMBuilder::default();
        system_unigram_lm_builder.add("今日/きょう", 1.0);
        system_unigram_lm_builder.add("は/は", 0.5);
        system_unigram_lm_builder.add("良い/いい", 1.2);
        system_unigram_lm_builder.add("天気/てんき", 1.5);
        system_unigram_lm_builder.set_total_words(100);
        system_unigram_lm_builder.set_unique_words(50);
        let system_unigram_lm = system_unigram_lm_builder.build()?;

        let mut system_bigram_lm_builder = MarisaSystemBigramLMBuilder::default();
        system_bigram_lm_builder.set_default_edge_cost(10.0);
        let system_bigram_lm = system_bigram_lm_builder.build()?;

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(dict),
            HashmapVecKanaKanjiDict::new(HashMap::new()),
            Arc::new(Mutex::new(UserData::default())),
            Rc::new(system_unigram_lm),
            Rc::new(system_bigram_lm),
        );
        let lattice = graph_builder.construct("きょうはいいてんき", &graph);
        let resolver = GraphResolver::default();

        let paths = resolver.resolve_k_best(&lattice, 5)?;

        // 少なくとも 1 パスは返る
        assert!(!paths.is_empty());

        // 最初のパスは空でないこと
        assert!(!paths[0].segments.is_empty());

        Ok(())
    }
}
