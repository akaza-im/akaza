use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use libakaza::corpus::{read_corpus_file, FullAnnotationCorpus};
use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::kana_kanji_dict::KanaKanjiDict;
use libakaza::kana_trie::marisa_kana_trie::MarisaKanaTrie;
use libakaza::lm::base::{SystemBigramLM, SystemUnigramLM};
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;
use libakaza::user_side_data::user_data::UserData;

use crate::utils::get_file_list;

pub struct OnMemorySystemUnigramLM {
    // word -> (word_id, cost)
    unigram_cost_map: Rc<RefCell<HashMap<String, (i32, f32)>>>,
}

impl OnMemorySystemUnigramLM {
    fn update(&self, word: &str, cost: f32) {
        let Some((word_id, _)) = self.find(word) else {
            // 登録されてない単語は無視。
            return;
        };

        self.unigram_cost_map
            .borrow_mut()
            .insert(word.to_string(), (word_id, cost));
    }
}

impl SystemUnigramLM for OnMemorySystemUnigramLM {
    fn get_default_cost(&self) -> f32 {
        20_f32
    }

    fn get_default_cost_for_short(&self) -> f32 {
        19_f32
    }

    fn find(&self, word: &str) -> Option<(i32, f32)> {
        self.unigram_cost_map.borrow().get(word).copied()
    }

    fn as_id_map(&self) -> HashMap<String, i32> {
        self.unigram_cost_map
            .borrow()
            .iter()
            .map(|(k, (id, _))| (k.clone(), *id))
            .collect()
    }

    fn as_hash_map(&self) -> HashMap<String, (i32, f32)> {
        self.unigram_cost_map.borrow().clone()
    }
}

pub struct OnMemorySystemBigramLM {
    // (word_id, word_id) -> cost
    bigram_cost: Rc<RefCell<HashMap<(i32, i32), f32>>>,
}

impl SystemBigramLM for OnMemorySystemBigramLM {
    fn get_default_edge_cost(&self) -> f32 {
        20_f32
    }

    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        self.bigram_cost
            .borrow()
            .get(&(word_id1, word_id2))
            .cloned()
    }
}

impl OnMemorySystemBigramLM {
    pub fn update(&self, word_id1: i32, word_id2: i32, cost: f32) {
        self.bigram_cost
            .borrow_mut()
            .insert((word_id1, word_id2), cost);
    }
}

/// 構造化パーセプトロンの学習を行います。
/// 構造化パーセプトロンは、シンプルな実装で、そこそこのパフォーマンスがでる(予定)
/// 構造化パーセプトロンでいい感じに動くようならば、構造化SVMなどに挑戦したい。
pub fn learn_structured_perceptron(src_dir: &String, epochs: i32) -> anyhow::Result<()> {
    // ここでは内部クラスなどを触ってスコア調整をしていかないといけないので、AkazaBuilder は使えない。

    let system_kana_kanji_dict = KanaKanjiDict::load("data/system_dict.trie")?;
    let all_yomis = system_kana_kanji_dict.all_yomis().unwrap();
    let system_kana_trie = MarisaKanaTrie::build(all_yomis);
    let segmenter = Segmenter::new(vec![Box::new(system_kana_trie)]);
    let system_single_term_dict = KanaKanjiDict::load("data/single_term.trie")?;

    let bigram_cost: Rc<RefCell<HashMap<(i32, i32), f32>>> = Rc::new(RefCell::new(HashMap::new()));
    let system_bigram_lm = Rc::new(OnMemorySystemBigramLM { bigram_cost });

    let real_system_unigram_lm = MarisaSystemUnigramLM::load("data/stats-vibrato-unigram.trie")?;

    let unigram_cost: Rc<RefCell<HashMap<String, (i32, f32)>>> =
        Rc::new(RefCell::new(real_system_unigram_lm.as_hash_map()));
    let system_unigram_lm = Rc::new(OnMemorySystemUnigramLM {
        unigram_cost_map: unigram_cost.clone(),
    });

    let mut graph_builder = GraphBuilder::new(
        system_kana_kanji_dict,
        system_single_term_dict,
        Arc::new(Mutex::new(UserData::default())),
        system_unigram_lm.clone(),
        system_bigram_lm.clone(),
    );

    for _ in 1..epochs {
        for file in get_file_list(Path::new(src_dir))? {
            let corpuses = read_corpus_file(file.as_path())?;
            for teacher in corpuses.iter() {
                learn(
                    teacher,
                    &segmenter,
                    &mut graph_builder,
                    system_unigram_lm.clone(),
                    system_bigram_lm.clone(),
                )?;
            }
        }
    }

    Ok(())
}

pub fn learn(
    teacher: &FullAnnotationCorpus,
    segmenter: &Segmenter,
    graph_builder: &mut GraphBuilder<OnMemorySystemUnigramLM, OnMemorySystemBigramLM>,
    system_unigram_lm: Rc<OnMemorySystemUnigramLM>,
    system_bigram_lm: Rc<OnMemorySystemBigramLM>,
) -> anyhow::Result<()> {
    // let system_kana_kanji_dict = KanaKanjiDictBuilder::default()
    //     .add("せんたくもの", "洗濯物")
    //     .add("せんたく", "選択/洗濯")
    //     .add("もの", "Mono")
    //     .add("ほす", "干す/HOS")
    //     .add("めんどう", "面倒")
    //     .build();

    let mut correct_edges: HashSet<String> = HashSet::new();
    if teacher.nodes.len() > 1 {
        for i in 1..teacher.nodes.len() {
            let key1 = teacher.nodes[i].key();
            let key2 = teacher.nodes[i].key();
            correct_edges.insert(key1 + "\t" + key2.as_str());
        }
    }

    let correct_nodes = teacher.correct_node_set();
    let yomi = teacher.yomi();
    let segmentation_result = segmenter.build(&yomi, None);
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
                    // info!("CORRECT: {:?}", node);
                    -1_f32
                } else {
                    1_f32
                };
                let (_, cost) = system_unigram_lm
                    .find(&node.key().to_string())
                    .unwrap_or((-1, 0_f32));
                system_unigram_lm.update(node.key().as_str(), cost + modifier);
            }
            for j in 1..nodes.len() {
                let word1 = &nodes[j - 1];
                let word2 = &nodes[j];
                let Some((word_id1, _)) = system_unigram_lm.find(&word1.key().to_string()) else {
                    // info!("{} is not registered in the real system unigram LM.",word1);
                    continue;
                };
                let Some((word_id2, _)) = system_unigram_lm.find(&word2.key().to_string()) else {
                    // info!("{} is not registered in the real system unigram LM.",word1);
                    continue;
                };

                let modifier = if correct_edges
                    .contains((word1.key().to_string() + "\t" + word2.key().as_str()).as_str())
                {
                    // info!("EDGE HIT {},{}", word1, word2);
                    -1_f32
                } else {
                    // info!("EDGE MISS {},{}", word1, word2);
                    1_f32
                };

                let v = system_bigram_lm
                    .get_edge_cost(word_id1, word_id2)
                    .unwrap_or(0_f32);

                system_bigram_lm.update(word_id1, word_id2, v + modifier);
            }
        }
    }
    // let dot = lattice.dump_cost_dot();
    // BufWriter::new(File::create("/tmp/dump.dot")?).write_fmt(format_args!("{}", dot))?;
    // println!("{:?}", unigram_cost);
    println!("{}", result);
    Ok(())
}
