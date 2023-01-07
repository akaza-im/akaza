use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use log::{info, warn};

use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::graph::word_node::WordNode;
use libakaza::kana_kanji_dict::KanaKanjiDict;
use libakaza::kana_trie::marisa_kana_trie::MarisaKanaTrie;
use libakaza::lm::system_bigram::SystemBigramLMBuilder;
use libakaza::lm::system_unigram_lm::SystemUnigramLMBuilder;
use libakaza::user_side_data::user_data::UserData;

/// 構造化パーセプトロンの学習を行います。
/// 構造化パーセプトロンは、シンプルな実装で、そこそこのパフォーマンスがでる(予定)
/// 構造化パーセプトロンでいい感じに動くようならば、構造化SVMなどに挑戦したい。
pub fn learn_structured_perceptron() -> anyhow::Result<()> {
    // ここでは内部クラスなどを触ってスコア調整をしていかないといけないので、AkazaBuilder は使えない。

    let mut unigram_cost: HashMap<String, f32> = HashMap::new();
    for _ in 1..20 {
        for teacher_src in &[
            "洗濯物/せんたくもの を/を 干す/ほす の/の が/が 面倒/めんどう だ/だ",
            "構造化/こうぞうか パーセプトロン/ぱーせぷとろん 面白い/おもしろい",
        ] {
            learn(teacher_src, &mut unigram_cost)?;
        }
    }

    Ok(())
}

pub fn learn(teacher_src: &str, unigram_cost: &mut HashMap<String, f32>) -> anyhow::Result<()> {
    let system_kana_kanji_dict = KanaKanjiDict::load("data/system_dict.trie")?;
    // let system_kana_kanji_dict = KanaKanjiDictBuilder::default()
    //     .add("せんたくもの", "洗濯物")
    //     .add("せんたく", "選択/洗濯")
    //     .add("もの", "Mono")
    //     .add("ほす", "干す/HOS")
    //     .add("めんどう", "面倒")
    //     .build();
    let system_single_term_dict = KanaKanjiDict::load("data/single_term.trie")?;

    let all_yomis = system_kana_kanji_dict.all_yomis().unwrap();
    let system_kana_trie = MarisaKanaTrie::build(all_yomis);
    let segmenter = Segmenter::new(vec![Box::new(system_kana_trie)]);
    let force_ranges: Vec<Range<usize>> = Vec::new();

    let mut unigram_lm_builder = SystemUnigramLMBuilder::default();
    for (key, cost) in unigram_cost.iter() {
        warn!("SYSTEM UNIGRM LM: {} cost={}", key.as_str(), *cost);
        unigram_lm_builder.add(key.as_str(), *cost);
    }

    let system_unigram_lm = unigram_lm_builder.build();
    let system_bigram_lm = SystemBigramLMBuilder::default().build();

    let teacher = Teacher::new(teacher_src);
    let correct_nodes = teacher.correct_node_set();
    let yomi = teacher.yomi();
    let segmentation_result = segmenter.build(&yomi, &force_ranges);
    let graph_builder = GraphBuilder::new(
        system_kana_kanji_dict,
        system_single_term_dict,
        Arc::new(Mutex::new(UserData::default())),
        Rc::new(system_unigram_lm),
        Rc::new(system_bigram_lm),
        0_f32,
        0_f32,
    );
    let graph_resolver = GraphResolver::default();

    let lattice = graph_builder.construct(yomi.as_str(), segmentation_result);
    let got = graph_resolver.resolve(&lattice)?;
    let terms: Vec<String> = got.iter().map(|f| f[0].kanji.clone()).collect();
    let result = terms.join("");

    if result != yomi {
        // エポックのたびに作りなおさないといけないオブジェクトが多すぎてごちゃごちゃしている。
        for i in 1..yomi.len() + 2 {
            // いったん、全部のノードのコストを1ずつ下げる
            let Some(nodes) = &lattice.node_list(i as i32) else {
                continue;
            };
            for node in *nodes {
                let modifier = if correct_nodes.contains(node) {
                    info!("CORRECT: {:?}", node);
                    -1_f32
                } else {
                    1_f32
                };
                let v = unigram_cost.get(&node.key().to_string()).unwrap_or(&0_f32);
                unigram_cost.insert(node.key(), *v + modifier);
            }

            // TODO エッジコストも考慮する
        }
    }
    // let dot = lattice.dump_cost_dot();
    // BufWriter::new(File::create("/tmp/dump.dot")?).write_fmt(format_args!("{}", dot))?;
    // println!("{:?}", unigram_cost);
    println!("{}", result);
    Ok(())
}

/// 教師データ
pub struct Teacher {
    pub nodes: Vec<WordNode>,
}

impl Teacher {
    /// 教師データをパースする。
    pub fn new(src: &str) -> Teacher {
        let p: Vec<&str> = src.split(' ').collect();
        let mut start_pos = 0;
        let mut nodes: Vec<WordNode> = Vec::new();
        for x in p {
            let (surface, yomi) = x.split_once('/').unwrap();
            nodes.push(WordNode::new(start_pos, surface, yomi));
            start_pos += yomi.len() as i32;
        }
        Teacher { nodes }
    }

    /// 教師データの「よみ」を返す。
    pub fn yomi(&self) -> String {
        let mut buf = String::new();
        for yomi in self.nodes.iter().map(|f| f.yomi.as_str()) {
            buf += yomi;
        }
        buf
    }

    /// 正解ノードを返す
    pub fn correct_node_set(&self) -> HashSet<WordNode> {
        HashSet::from_iter(self.nodes.iter().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d() -> anyhow::Result<()> {
        learn_structured_perceptron()
    }
}
