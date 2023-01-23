use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use anyhow::Context;
use log::{info, trace};

use crate::graph::candidate::Candidate;
use crate::graph::lattice_graph::LatticeGraph;
use crate::graph::word_node::WordNode;
use crate::lm::base::{SystemBigramLM, SystemUnigramLM};

/**
 * Segmenter により分割されたかな表現から、グラフを構築する。
 */
#[derive(Default)]
pub struct GraphResolver {}

impl GraphResolver {
    /**
     * ビタビアルゴリズムで最適な経路を見つける。
     */
    pub fn resolve<U: SystemUnigramLM, B: SystemBigramLM>(
        &self,
        lattice: &LatticeGraph<U, B>,
    ) -> anyhow::Result<Vec<Vec<Candidate>>> {
        let yomi = &lattice.yomi;
        let mut prevmap: HashMap<&WordNode, &WordNode> = HashMap::new();
        let mut costmap: HashMap<&WordNode, f32> = HashMap::new();

        // 前向きに動的計画法でたどる
        for i in 1..yomi.len() + 2 {
            let Some(nodes) = &lattice.node_list(i as i32) else {
                continue;
            };
            for node in *nodes {
                let node_cost = lattice.get_node_cost(node);
                trace!("kanji={}, Cost={}", node, node_cost);
                let mut cost = f32::MAX;
                let mut shortest_prev = None;
                let prev_nodes = lattice.get_prev_nodes(node).with_context(|| {
                    format!(
                        "Cannot get prev nodes for '{}' start={} lattice={:?}",
                        node.surface, node.start_pos, lattice
                    )
                })?;
                for prev in prev_nodes {
                    let edge_cost = lattice.get_edge_cost(prev, node);
                    let prev_cost = costmap.get(prev).unwrap_or(&0_f32); // unwrap が必要なのは、 __BOS__ 用。
                    let tmp_cost = prev_cost + edge_cost + node_cost;
                    trace!(
                        "Replace??? prev_cost={} tmp_cost={} < cost={}: {}",
                        prev_cost,
                        tmp_cost,
                        cost,
                        prev
                    );
                    // コストが最小な経路を選ぶようにする。
                    // そういうふうにコストを付与しているので。
                    if cost > tmp_cost {
                        if shortest_prev.is_none() {
                            trace!("Replace None by {}", prev);
                        } else {
                            trace!("Replace {} by {}", shortest_prev.unwrap(), prev);
                        }
                        cost = tmp_cost;
                        shortest_prev = Some(prev);
                    }
                }
                prevmap.insert(node, shortest_prev.unwrap());
                costmap.insert(node, cost);
            }
        }

        // 後ろ向きに候補を探していく
        let eos = lattice
            .get((yomi.len() + 1) as i32)
            .unwrap()
            .get(0)
            .unwrap();
        let bos = lattice.get(0).unwrap().get(0).unwrap();
        let mut node = eos;
        let mut result: Vec<Vec<Candidate>> = Vec::new();
        while node != bos {
            if node.surface != "__EOS__" {
                // 同一の開始位置、終了位置を持つものを集める。
                let end_pos = node.start_pos + (node.yomi.len() as i32);
                let candidates: Vec<Candidate> =
                    self.get_candidates(node, lattice, &costmap, end_pos);
                result.push(candidates);
            }
            node = prevmap
                .get(node)
                .unwrap_or_else(|| panic!("Cannot get previous node: {}", node.surface));
        }
        result.reverse();
        Ok(result)
    }

    fn get_candidates<U: SystemUnigramLM, B: SystemBigramLM>(
        &self,
        node: &WordNode,
        lattice: &LatticeGraph<U, B>,
        costmap: &HashMap<&WordNode, f32>,
        end_pos: i32,
    ) -> Vec<Candidate> {
        // end_pos で終わる単語を得る。
        let mut strict_results: Vec<Candidate> = lattice
            .node_list(end_pos)
            .unwrap()
            .iter()
            .filter(|alt_node| {
                alt_node.start_pos == node.start_pos // 同じ位置かそれより前から始まっている
                    && alt_node.yomi.len() == node.yomi.len() // 同じ長さの単語を得る
            })
            .map(|f| Candidate {
                surface: f.surface.clone(),
                yomi: f.yomi.clone(),
                cost: *costmap.get(f).unwrap(),
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
        cost_map: &&HashMap<&WordNode, f32>,
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

        let Some(targets) = lattice
            .node_list(end_pos) else {
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
            .map(|f| BreakDown {
                node: f.clone(),
                head_cost: (*cost_map.get(f).unwrap()), // 先頭から辿った場合のコスト
                tail_cost: tail_cost
                    + lattice.get_node_cost(f)
                    + next_node
                        .map(|nn| lattice.get_edge_cost(f, nn))
                        .unwrap_or_else(|| lattice.get_default_edge_cost()),
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
                panic!("??? underflow: {:?}, {:?}", required_len, target.node.yomi);
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
        (self.head_cost + self.tail_cost).partial_cmp(&(other.head_cost + other.tail_cost))
    }
}

impl Ord for BreakDown {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.head_cost + self.tail_cost)
            .partial_cmp(&(other.head_cost + other.tail_cost))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::btree_map::BTreeMap;
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
            .set_default_cost(20_f32)
            .set_default_cost_for_short(19_f32)
            .build();
        let system_bigram_lm = MarisaSystemBigramLMBuilder::default()
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
            .set_default_cost(19_f32)
            .set_default_cost_for_short(20_f32)
            .build();
        let system_bigram_lm = MarisaSystemBigramLMBuilder::default()
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
            .set_default_cost(19_f32)
            .set_default_cost_for_short(20_f32)
            .build();
        let system_bigram_lm = MarisaSystemBigramLMBuilder::default()
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
}
