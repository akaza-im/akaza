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
use libakaza::lm::on_memory::on_memory_system_bigram_lm::OnMemorySystemBigramLM;
use libakaza::lm::on_memory::on_memory_system_unigram_lm::OnMemorySystemUnigramLM;
use libakaza::lm::system_bigram::{MarisaSystemBigramLM, MarisaSystemBigramLMBuilder};
use libakaza::lm::system_unigram_lm::{MarisaSystemUnigramLM, MarisaSystemUnigramLMBuilder};
use libakaza::user_side_data::user_data::UserData;

struct LearningService {
    graph_builder: GraphBuilder<OnMemorySystemUnigramLM, OnMemorySystemBigramLM>,
    segmenter: Segmenter,
    system_unigram_lm: Rc<OnMemorySystemUnigramLM>,
    system_bigram_lm: Rc<OnMemorySystemBigramLM>,
}

impl LearningService {
    pub fn new(src_unigram: &str, src_bigram: &str, corpuses: &[&str]) -> anyhow::Result<Self> {
        let system_kana_kanji_dict = KanaKanjiDict::load("data/system_dict.trie")?;
        let all_yomis = system_kana_kanji_dict.all_yomis().unwrap();
        let system_kana_trie = MarisaKanaTrie::build(all_yomis);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(system_kana_trie))]);
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
            for fname in corpuses {
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
        let system_unigram_lm = Rc::new(OnMemorySystemUnigramLM::new(
            Rc::new(RefCell::new(unigram_map)),
            src_system_unigram_lm.get_default_cost(),
            src_system_unigram_lm.get_default_cost_for_short(),
        ));

        info!("bigram source file: {}", src_bigram);
        let src_system_bigram_lm = MarisaSystemBigramLM::load(src_bigram)?;
        let system_bigram_lm = Rc::new(OnMemorySystemBigramLM::new(
            Rc::new(RefCell::new(src_system_bigram_lm.as_hash_map())),
            src_system_bigram_lm.get_default_edge_cost(),
        ));

        let graph_builder = GraphBuilder::new(
            system_kana_kanji_dict,
            system_single_term_dict,
            Arc::new(Mutex::new(UserData::default())),
            system_unigram_lm.clone(),
            system_bigram_lm.clone(),
        );

        Ok(LearningService {
            graph_builder,
            segmenter,
            system_unigram_lm,
            system_bigram_lm,
        })
    }

    pub fn try_learn(&self, epochs: i32, delta: f32, corpus: &str) -> anyhow::Result<()> {
        let corpuses = read_corpus_file(Path::new(corpus))?;
        for _ in 1..epochs {
            let mut ok_cnt = 0;
            for teacher in corpuses.iter() {
                let succeeded = self.learn(delta, teacher)?;

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

    pub fn learn(&self, delta: f32, teacher: &FullAnnotationCorpus) -> anyhow::Result<bool> {
        let yomi = teacher.yomi();
        let surface = teacher.surface();
        let segmentation_result = self.segmenter.build(&yomi, None);
        let graph_resolver = GraphResolver::default();

        let lattice = self
            .graph_builder
            .construct(yomi.as_str(), segmentation_result);
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
                    let (_, cost) = self
                        .system_unigram_lm
                        .find(&key.to_string())
                        .unwrap_or((-1, 0_f32));
                    self.system_unigram_lm.update(key.as_str(), cost - delta);
                }
            }

            // learn bigram
            if teacher.nodes.len() > 1 {
                for i in 1..teacher.nodes.len() {
                    let key1 = teacher.nodes[i - 1].key();
                    let key2 = teacher.nodes[i].key();
                    let Some((word_id1, _)) = self.system_unigram_lm.find(key1.as_str()) else {
                        // info!("{} is not registered in the real system unigram LM.",word1);
                        continue;
                    };
                    let Some((word_id2, _)) = self.system_unigram_lm.find(key2.as_str()) else {
                        // info!("{} is not registered in the real system unigram LM.",word1);
                        continue;
                    };
                    let v = self
                        .system_bigram_lm
                        .get_edge_cost(word_id1, word_id2)
                        .unwrap_or(0_f32);
                    info!(
                        "Update bigram cost: {}={},{}={}, v={}",
                        key1, word_id1, key2, word_id2, v
                    );
                    self.system_bigram_lm.update(word_id1, word_id2, v - delta);
                }
            }

            debug!("BAD! result={}, surface={}", result, surface);
            Ok(false)
        } else {
            debug!("学習完了! result={}", result);
            Ok(true)
        }
    }

    pub fn save_unigram(&self, dst_unigram: &str) -> anyhow::Result<()> {
        // unigram
        let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
        for (key, (_, cost)) in self.system_unigram_lm.as_hash_map() {
            unigram_builder.add(key.as_str(), cost);
        }
        // ↓本来なら現在のデータで再調整すべきだが、一旦元のものを使う。
        // TODO あとで整理する
        unigram_builder.set_default_cost(self.system_unigram_lm.get_default_cost());
        unigram_builder
            .set_default_cost_for_short(self.system_unigram_lm.get_default_cost_for_short());
        info!("Save unigram to {}", dst_unigram);
        unigram_builder.save(dst_unigram)?;
        Ok(())
    }

    pub fn save_bigram(&self, dst_unigram: &str, dst_bigram: &str) -> anyhow::Result<()> {
        // bigram の保存
        let new_unigram = MarisaSystemUnigramLM::load(dst_unigram)?;
        let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
        let srcmap = self.system_unigram_lm.as_hash_map();
        let src_wordid2key = srcmap
            .iter()
            .map(|(key, (word_id, _))| (*word_id, key.to_string()))
            .collect::<HashMap<i32, String>>();
        // info!("src_wordid2key: {:?}", src_wordid2key);
        for ((word_id1, word_id2), cost) in self.system_bigram_lm.as_hash_map() {
            // このへんで落ちるときはデータの整合性がとれてないことがあるので、work/ 以下のデータを一度全部作り直した方が
            // 良いケースが多いです。work/ 以下を作り直すと良いです。

            // KNOWN BUG:
            // Unknown word_id が一種類出ます。が、なぜ出るのか不明。
            // 一個ぐらいのデータがロストしてもここでは問題がないので後回し。

            let Some(word1) = src_wordid2key
                .get(&word_id1) else {
                info!("Unknown word_id: {}", word_id1);
                continue;
            };
            let Some((new_word_id1, _)) = new_unigram
                .find(word1) else {
                info!("Unknown word: {}", word1);
                continue;
            };

            let Some(word2) = src_wordid2key
                .get(&word_id2) else {
                info!("Unknown word_id: {}", word_id2);
                continue;
            };

            let Some((new_word_id2, _)) = new_unigram.find(word2) else {
                info!("Unknown word: {}", word2);
                continue;
            };
            bigram_builder.add(new_word_id1, new_word_id2, cost);
        }
        // ↓本来なら現在のデータで再調整すべきだが、一旦元のものを使う。
        // TODO あとで整理する
        bigram_builder.set_default_edge_cost(self.system_bigram_lm.get_default_edge_cost());
        info!("Save bigram to {}", dst_bigram);
        bigram_builder.save(dst_bigram)?;

        Ok(())
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
    let service = LearningService::new(
        src_unigram,
        src_bigram,
        &[may_corpus, should_corpus, must_corpus],
    )?;

    // 実際の学習をさせる
    for (epoch, corpus) in [
        (may_epochs, may_corpus),
        (should_epochs, should_corpus),
        (must_epochs, must_corpus),
    ] {
        service.try_learn(epoch, delta, corpus)?;
    }

    // 保存していく
    service.save_unigram(dst_unigram)?;
    service.save_bigram(dst_unigram, dst_bigram)?;

    Ok(())
}
