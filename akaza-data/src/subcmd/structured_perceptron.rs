use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use libakaza::corpus::{read_corpus_file, FullAnnotationCorpus};
use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::kana_kanji_dict::KanaKanjiDict;
use libakaza::kana_trie::marisa_kana_trie::MarisaKanaTrie;
use libakaza::lm::system_bigram::SystemBigramLMBuilder;
use libakaza::lm::system_unigram_lm::{SystemUnigramLM, SystemUnigramLMBuilder};
use libakaza::user_side_data::user_data::UserData;

use crate::utils::get_file_list;

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
    let system_bigram_lm = SystemBigramLMBuilder::default().build();
    let real_system_unigram_lm = SystemUnigramLM::load("data/stats-vibrato-unigram.trie")?;
    let mut graph_builder = GraphBuilder::new(
        system_kana_kanji_dict,
        system_single_term_dict,
        Arc::new(Mutex::new(UserData::default())),
        Rc::new(SystemUnigramLMBuilder::default().build()),
        Rc::new(system_bigram_lm),
        0_f32,
        0_f32,
    );

    let mut unigram_cost: HashMap<String, f32> = HashMap::new();
    let mut bigram_cost: HashMap<(i32, i32), f32> = HashMap::new();

    for _ in 1..epochs {
        for file in get_file_list(Path::new(src_dir))? {
            let corpuses = read_corpus_file(file.as_path())?;
            for teacher in corpuses.iter() {
                learn(
                    teacher,
                    &mut unigram_cost,
                    &mut bigram_cost,
                    &segmenter,
                    &mut graph_builder,
                    &real_system_unigram_lm,
                )?;
            }
        }
    }

    Ok(())
}

pub fn learn(
    teacher: &FullAnnotationCorpus,
    unigram_cost: &mut HashMap<String, f32>,
    bigram_cost: &mut HashMap<(i32, i32), f32>,
    segmenter: &Segmenter,
    graph_builder: &mut GraphBuilder,
    real_system_unigram_lm: &SystemUnigramLM,
) -> anyhow::Result<()> {
    // let system_kana_kanji_dict = KanaKanjiDictBuilder::default()
    //     .add("せんたくもの", "洗濯物")
    //     .add("せんたく", "選択/洗濯")
    //     .add("もの", "Mono")
    //     .add("ほす", "干す/HOS")
    //     .add("めんどう", "面倒")
    //     .build();

    let force_ranges: Vec<Range<usize>> = Vec::new();

    let mut unigram_lm_builder = SystemUnigramLMBuilder::default();
    for (key, cost) in unigram_cost.iter() {
        // warn!("SYSTEM UNIGRM LM: {} cost={}", key.as_str(), *cost);
        unigram_lm_builder.add(key.as_str(), *cost);
    }
    let system_unigram_lm = unigram_lm_builder.build();

    let mut bigram_lm_builder = SystemBigramLMBuilder::default();
    for ((word_id1, word_id2), cost) in bigram_cost.iter() {
        bigram_lm_builder.add(*word_id1, *word_id2, *cost);
    }
    let system_bigram_lm = bigram_lm_builder.build();

    graph_builder.set_system_unigram_lm(Rc::new(system_unigram_lm));
    graph_builder.set_system_bigram_lm(Rc::new(system_bigram_lm));

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
    let segmentation_result = segmenter.build(&yomi, &force_ranges);
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
                let v = unigram_cost.get(&node.key().to_string()).unwrap_or(&0_f32);
                unigram_cost.insert(node.key(), *v + modifier);
            }
            for j in 1..nodes.len() {
                let word1 = &nodes[j - 1];
                let word2 = &nodes[j];
                let Some((word_id1, _)) = real_system_unigram_lm.find(&word1.key().to_string()) else {
                    // info!("{} is not registered in the real system unigram LM.",word1);
                    continue;
                };
                let Some((word_id2, _)) = real_system_unigram_lm.find(&word2.key().to_string()) else {
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

                let v = bigram_cost.get(&(word_id1, word_id2)).unwrap_or(&0_f32);

                bigram_cost.insert((word_id1, word_id2), *v + modifier);
            }
        }
    }
    // let dot = lattice.dump_cost_dot();
    // BufWriter::new(File::create("/tmp/dump.dot")?).write_fmt(format_args!("{}", dot))?;
    // println!("{:?}", unigram_cost);
    println!("{}", result);
    Ok(())
}

