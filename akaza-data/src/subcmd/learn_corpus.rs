use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use log::{debug, info};

use libakaza::corpus::{read_corpus_file, FullAnnotationCorpus};
use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::kana_kanji_dict::KanaKanjiDict;
use libakaza::kana_trie::marisa_kana_trie::MarisaKanaTrie;
use libakaza::lm::base::{SystemBigramLM, SystemUnigramLM};
use libakaza::lm::system_bigram::{MarisaSystemBigramLM, MarisaSystemBigramLMBuilder};
use libakaza::lm::system_unigram_lm::{MarisaSystemUnigramLM, MarisaSystemUnigramLMBuilder};
use libakaza::user_side_data::user_data::UserData;

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

    fn as_hash_map(&self) -> HashMap<(i32, i32), f32> {
        self.bigram_cost.borrow().clone()
    }
}

impl OnMemorySystemBigramLM {
    pub fn update(&self, word_id1: i32, word_id2: i32, cost: f32) {
        self.bigram_cost
            .borrow_mut()
            .insert((word_id1, word_id2), cost);
    }
}

/// コーパスを元にした学習を行います。
#[allow(clippy::too_many_arguments)]
pub fn learn_corpus(
    delta: f32,
    may_epochs: i32,
    should_epochs: i32,
    must_epochs: i32,
    may_corpus: &str,
    should_corpus: &str,
    must_corpus: &str,
    src_unigram: &str,
    src_bigram: &str,
    dst_unigram: &str,
    dst_bigram: &str,
) -> anyhow::Result<()> {
    // ここでは内部クラスなどを触ってスコア調整をしていかないといけないので、AkazaBuilder は使えない。

    let system_kana_kanji_dict = KanaKanjiDict::load("data/system_dict.trie")?;
    let all_yomis = system_kana_kanji_dict.all_yomis().unwrap();
    let system_kana_trie = MarisaKanaTrie::build(all_yomis);
    let segmenter = Segmenter::new(vec![Box::new(system_kana_trie)]);
    let system_single_term_dict = KanaKanjiDict::load("data/single_term.trie")?;

    info!("unigram source file: {}", src_unigram);
    let src_system_unigram_lm = MarisaSystemUnigramLM::load(src_unigram)?;
    let mut unigram_map = src_system_unigram_lm.as_hash_map();
    // unigram trie に登録されていない単語を登録していく。
    {
        let mut max_id = *unigram_map
            .iter()
            .map(|(_, (id, _))| id)
            .max()
            .unwrap_or(&0);
        for fname in [may_corpus, should_corpus, must_corpus] {
            let corpuses = read_corpus_file(Path::new(fname))?;
            for corpus in corpuses {
                for node in corpus.nodes {
                    if !unigram_map.contains_key(node.key().as_str()) {
                        info!(
                            "Insert missing element: {} max_id={}",
                            node.key(),
                            max_id + 1
                        );
                        unigram_map.insert(
                            node.key(),
                            (max_id + 1, src_system_unigram_lm.get_default_cost()),
                        );
                        max_id += 1;
                    }
                }
            }
        }
    }
    let unigram_cost: Rc<RefCell<HashMap<String, (i32, f32)>>> = Rc::new(RefCell::new(unigram_map));
    let system_unigram_lm = Rc::new(OnMemorySystemUnigramLM {
        unigram_cost_map: unigram_cost,
    });

    info!("bigram source file: {}", src_bigram);
    let src_system_bigram_lm = MarisaSystemBigramLM::load(src_bigram)?;
    let src_system_bigram_lm_map = src_system_bigram_lm.as_hash_map();
    let bigram_cost: Rc<RefCell<HashMap<(i32, i32), f32>>> =
        Rc::new(RefCell::new(src_system_bigram_lm_map));
    let system_bigram_lm = Rc::new(OnMemorySystemBigramLM { bigram_cost });

    let mut graph_builder = GraphBuilder::new(
        system_kana_kanji_dict,
        system_single_term_dict,
        Arc::new(Mutex::new(UserData::default())),
        system_unigram_lm.clone(),
        system_bigram_lm.clone(),
    );

    // 実際の学習をさせる
    for (epoch, corpus) in [
        (may_epochs, may_corpus),
        (should_epochs, should_corpus),
        (must_epochs, must_corpus),
    ] {
        run_learning(
            epoch,
            delta,
            corpus,
            &segmenter,
            &mut graph_builder,
            system_unigram_lm.clone(),
            system_bigram_lm.clone(),
        )?;
    }

    // 保存していく
    {
        // unigram
        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        for (key, (_, cost)) in system_unigram_lm.as_hash_map() {
            unigram_builder.add(key.as_str(), cost);
        }
        unigram_builder.set_default_cost(src_system_unigram_lm.get_default_cost());
        unigram_builder
            .set_default_cost_for_short(src_system_unigram_lm.get_default_cost_for_short());
        info!("Save unigram to {}", dst_unigram);
        unigram_builder.save(dst_unigram)?;
    }
    {
        // bigram の保存
        let new_unigram = MarisaSystemUnigramLM::load(dst_unigram)?;
        let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
        let srcmap = system_unigram_lm.as_hash_map();
        let src_wordid2key = srcmap
            .iter()
            .map(|(key, (word_id, _))| (*word_id, key.to_string()))
            .collect::<HashMap<i32, String>>();
        for ((word_id1, word_id2), cost) in system_bigram_lm.as_hash_map() {
            let (new_word_id1, _) =
                new_unigram
                    .find(src_wordid2key.get(&word_id1).unwrap_or_else(|| {
                        panic!("Missing word_id in src_wordid2key: {}", word_id1)
                    }))
                    .expect("Missing word_id in new_unigram");
            let (new_word_id2, _) = new_unigram
                .find(src_wordid2key.get(&word_id2).unwrap())
                .unwrap();
            bigram_builder.add(new_word_id1, new_word_id2, cost);
        }
        bigram_builder.set_default_edge_cost(src_system_bigram_lm.get_default_edge_cost());
        info!("Save bigram to {}", dst_bigram);
        bigram_builder.save(dst_bigram)?;
    }

    Ok(())
}

fn run_learning(
    epochs: i32,
    delta: f32,
    corpus: &str,
    segmenter: &Segmenter,
    graph_builder: &mut GraphBuilder<OnMemorySystemUnigramLM, OnMemorySystemBigramLM>,
    system_unigram_lm: Rc<OnMemorySystemUnigramLM>,
    system_bigram_lm: Rc<OnMemorySystemBigramLM>,
) -> anyhow::Result<()> {
    let corpuses = read_corpus_file(Path::new(corpus))?;
    for _ in 1..epochs {
        let mut ok_cnt = 0;
        for teacher in corpuses.iter() {
            let succeeded = learn(
                delta,
                teacher,
                segmenter,
                graph_builder,
                system_unigram_lm.clone(),
                system_bigram_lm.clone(),
            )?;

            if succeeded {
                ok_cnt += 1;
            }
        }
        info!("ok_cnt={} corpuses.len()={}", ok_cnt, corpuses.len());
        if ok_cnt == corpuses.len() {
            info!("Learning process finished.");
            break;
        }
    }

    Ok(())
}

pub fn learn(
    delta: f32,
    teacher: &FullAnnotationCorpus,
    segmenter: &Segmenter,
    graph_builder: &mut GraphBuilder<OnMemorySystemUnigramLM, OnMemorySystemBigramLM>,
    system_unigram_lm: Rc<OnMemorySystemUnigramLM>,
    system_bigram_lm: Rc<OnMemorySystemBigramLM>,
) -> anyhow::Result<bool> {
    let yomi = teacher.yomi();
    let surface = teacher.surface();
    let segmentation_result = segmenter.build(&yomi, None);
    let graph_resolver = GraphResolver::default();

    let lattice = graph_builder.construct(yomi.as_str(), segmentation_result);
    let got = graph_resolver.resolve(&lattice)?;
    let terms: Vec<String> = got.iter().map(|f| f[0].kanji.clone()).collect();
    let result = terms.join("");

    println!("{}", result);

    // 正解じゃないときには出現頻度の確率が正しくないということだと思いますんで
    // 頻度を増やす。
    if result != surface {
        // learn unigram
        if !teacher.nodes.is_empty() {
            for i in 0..teacher.nodes.len() {
                let key = teacher.nodes[i].key();
                let (_, cost) = system_unigram_lm
                    .find(&key.to_string())
                    .unwrap_or((-1, 0_f32));
                system_unigram_lm.update(key.as_str(), cost - delta);
            }
        }

        // learn bigram
        if teacher.nodes.len() > 1 {
            for i in 1..teacher.nodes.len() {
                let key1 = teacher.nodes[i - 1].key();
                let key2 = teacher.nodes[i].key();
                let Some((word_id1, _)) = system_unigram_lm.find(key1.as_str()) else {
                    // info!("{} is not registered in the real system unigram LM.",word1);
                    continue;
                };
                let Some((word_id2, _)) = system_unigram_lm.find(key2.as_str()) else {
                    // info!("{} is not registered in the real system unigram LM.",word1);
                    continue;
                };
                let v = system_bigram_lm
                    .get_edge_cost(word_id1, word_id2)
                    .unwrap_or(0_f32);
                system_bigram_lm.update(word_id1, word_id2, v - delta);
            }
        }

        debug!("BAD! result={}, surface={}", result, surface);
        Ok(false)
    } else {
        debug!("学習完了! result={}", result);
        Ok(true)
    }
}
