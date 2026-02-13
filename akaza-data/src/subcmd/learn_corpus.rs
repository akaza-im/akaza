use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use encoding_rs::UTF_8;
use log::{debug, info};

use crate::wordcnt::wordcnt_bigram::WordcntBigram;
use crate::wordcnt::wordcnt_skip_bigram::WordcntSkipBigram;
use crate::wordcnt::wordcnt_unigram::WordcntUnigram;
use libakaza::corpus::{read_corpus_file, FullAnnotationCorpus};
use libakaza::dict::skk::read::read_skkdict;
use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::graph::word_node::{BOS_TOKEN_KEY, EOS_TOKEN_KEY};
use libakaza::kana_kanji::hashmap_vec::HashmapVecKanaKanjiDict;
use libakaza::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use libakaza::lm::base::{SystemBigramLM, SystemSkipBigramLM, SystemUnigramLM};
use libakaza::lm::on_memory::on_memory_system_bigram_lm::OnMemorySystemBigramLM;
use libakaza::lm::on_memory::on_memory_system_skip_bigram_lm::OnMemorySystemSkipBigramLM;
use libakaza::lm::on_memory::on_memory_system_unigram_lm::OnMemorySystemUnigramLM;
use libakaza::lm::system_bigram::MarisaSystemBigramLMBuilder;
use libakaza::lm::system_skip_bigram::MarisaSystemSkipBigramLMBuilder;
use libakaza::lm::system_unigram_lm::{MarisaSystemUnigramLM, MarisaSystemUnigramLMBuilder};
use libakaza::user_side_data::user_data::UserData;

struct LearningService {
    graph_builder:
        GraphBuilder<OnMemorySystemUnigramLM, OnMemorySystemBigramLM, HashmapVecKanaKanjiDict>,
    segmenter: Segmenter,
    system_unigram_lm: Rc<OnMemorySystemUnigramLM>,
    system_bigram_lm: Rc<OnMemorySystemBigramLM>,
    system_skip_bigram_lm: Option<Rc<OnMemorySystemSkipBigramLM>>,
}

impl LearningService {
    pub fn new(
        src_unigram: &str,
        src_bigram: &str,
        src_skip_bigram: Option<&str>,
        corpuses: &[&str],
    ) -> anyhow::Result<Self> {
        let system_kana_kanji_dict = read_skkdict(Path::new("data/SKK-JISYO.akaza"), UTF_8)?;
        let all_yomis = system_kana_kanji_dict.keys().cloned().collect::<Vec<_>>();
        let system_kana_trie = CedarwoodKanaTrie::build(all_yomis);
        let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(system_kana_trie))]);

        info!("unigram source file: {}", src_unigram);
        let src_system_unigram_lm = WordcntUnigram::load(src_unigram)?;
        let mut unigram_map = src_system_unigram_lm.to_count_hashmap();
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
                            unigram_map.insert(node.key(), (max_id + 1, 1));
                            max_id += 1;
                        }
                    }
                }
            }
        }
        let system_unigram_lm = Rc::new(OnMemorySystemUnigramLM::new(
            Rc::new(RefCell::new(unigram_map)),
            src_system_unigram_lm.total_words,
            src_system_unigram_lm.unique_words,
        ));

        info!("bigram source file: {}", src_bigram);
        let src_system_bigram_lm = WordcntBigram::load(src_bigram)?;
        let system_bigram_lm = Rc::new(OnMemorySystemBigramLM::new(
            Rc::new(RefCell::new(src_system_bigram_lm.to_cnt_map())),
            src_system_bigram_lm.get_default_edge_cost(),
            src_system_bigram_lm.total_words,
            src_system_bigram_lm.unique_words,
        ));

        let system_skip_bigram_lm = if let Some(skip_bigram_path) = src_skip_bigram {
            info!("skip-bigram source file: {}", skip_bigram_path);
            let src_skip_bigram_lm = WordcntSkipBigram::load(skip_bigram_path)?;
            Some(Rc::new(OnMemorySystemSkipBigramLM::new(
                Rc::new(RefCell::new(src_skip_bigram_lm.to_cnt_map())),
                src_skip_bigram_lm.get_default_skip_cost(),
                src_skip_bigram_lm.total_words,
                src_skip_bigram_lm.unique_words,
            )))
        } else {
            None
        };

        let graph_builder = GraphBuilder::new(
            HashmapVecKanaKanjiDict::new(system_kana_kanji_dict),
            HashmapVecKanaKanjiDict::new(HashMap::default()),
            Arc::new(Mutex::new(UserData::default())),
            system_unigram_lm.clone(),
            system_bigram_lm.clone(),
        );

        Ok(LearningService {
            graph_builder,
            segmenter,
            system_unigram_lm,
            system_bigram_lm,
            system_skip_bigram_lm,
        })
    }

    pub fn try_learn(&self, epochs: i32, step_size: f32, corpus: &str) -> anyhow::Result<()> {
        let corpuses = read_corpus_file(Path::new(corpus))?;
        for _ in 1..epochs {
            let mut ok_cnt = 0;
            for teacher in corpuses.iter() {
                let succeeded = self.learn(step_size, teacher)?;

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

    /// 予測パスからキー列を抽出する（"surface/yomi" 形式）
    fn extract_pred_keys(got: &[Vec<libakaza::graph::candidate::Candidate>]) -> Vec<String> {
        got.iter()
            .map(|candidates| format!("{}/{}", candidates[0].surface, candidates[0].yomi))
            .collect()
    }

    /// 教師パスからキー列を抽出する（"surface/yomi" 形式）
    fn extract_teacher_keys(teacher: &FullAnnotationCorpus) -> Vec<String> {
        teacher.nodes.iter().map(|node| node.key()).collect()
    }

    /// キー列から bigram の word_id ペアを抽出する（BOS/EOS 含む）
    fn extract_bigram_ids(&self, keys: &[String]) -> Vec<(i32, i32)> {
        let mut pairs = Vec::new();

        if keys.is_empty() {
            return pairs;
        }

        // BOS → 最初の単語
        if let Some((bos_id, _)) = self.system_unigram_lm.find(BOS_TOKEN_KEY) {
            if let Some((first_id, _)) = self.system_unigram_lm.find(&keys[0]) {
                pairs.push((bos_id, first_id));
            }
        }

        // 隣接ペア
        for i in 1..keys.len() {
            if let Some((id1, _)) = self.system_unigram_lm.find(&keys[i - 1]) {
                if let Some((id2, _)) = self.system_unigram_lm.find(&keys[i]) {
                    pairs.push((id1, id2));
                }
            }
        }

        // 最後の単語 → EOS
        if let Some((eos_id, _)) = self.system_unigram_lm.find(EOS_TOKEN_KEY) {
            if let Some((last_id, _)) = self.system_unigram_lm.find(keys.last().unwrap()) {
                pairs.push((last_id, eos_id));
            }
        }

        pairs
    }

    /// キー列から skip-bigram の word_id ペアを抽出する（i-2, i）
    fn extract_skip_bigram_ids(&self, keys: &[String]) -> Vec<(i32, i32)> {
        let mut pairs = Vec::new();

        if keys.len() <= 2 {
            return pairs;
        }

        for i in 2..keys.len() {
            if let Some((id1, _)) = self.system_unigram_lm.find(&keys[i - 2]) {
                if let Some((id2, _)) = self.system_unigram_lm.find(&keys[i]) {
                    pairs.push((id1, id2));
                }
            }
        }

        pairs
    }

    pub fn learn(&self, step_size: f32, teacher: &FullAnnotationCorpus) -> anyhow::Result<bool> {
        let yomi = teacher.yomi();
        let surface = teacher.surface();
        let segmentation_result = self.segmenter.build(&yomi, None);
        let graph_resolver = GraphResolver::default();

        let lattice = self
            .graph_builder
            .construct(yomi.as_str(), &segmentation_result);
        let got = graph_resolver.resolve(&lattice)?;
        let terms: Vec<String> = got.iter().map(|f| f[0].surface.clone()).collect();
        let result = terms.join("");

        println!("{result}");

        if result != surface {
            let pred_keys = Self::extract_pred_keys(&got);
            let teacher_keys = Self::extract_teacher_keys(teacher);

            // --- unigram 調整 ---
            // 教師パス: コスト減算（選ばれやすくする）
            for key in &teacher_keys {
                self.system_unigram_lm.adjust_cost(key, -step_size);
            }
            // 予測パス: コスト加算（選ばれにくくする）
            for key in &pred_keys {
                self.system_unigram_lm.adjust_cost(key, step_size);
            }

            // --- bigram 調整 ---
            let teacher_bigrams = self.extract_bigram_ids(&teacher_keys);
            let pred_bigrams = self.extract_bigram_ids(&pred_keys);

            for (id1, id2) in &teacher_bigrams {
                info!("Adjust bigram (teacher, -step): ({}, {})", id1, id2);
                self.system_bigram_lm.adjust_cost(*id1, *id2, -step_size);
            }
            for (id1, id2) in &pred_bigrams {
                info!("Adjust bigram (pred, +step): ({}, {})", id1, id2);
                self.system_bigram_lm.adjust_cost(*id1, *id2, step_size);
            }

            // --- skip-bigram 調整 ---
            if let Some(skip_bigram_lm) = &self.system_skip_bigram_lm {
                let teacher_skip = self.extract_skip_bigram_ids(&teacher_keys);
                let pred_skip = self.extract_skip_bigram_ids(&pred_keys);

                for (id1, id2) in &teacher_skip {
                    info!("Adjust skip-bigram (teacher, -step): ({}, {})", id1, id2);
                    skip_bigram_lm.adjust_cost(*id1, *id2, -step_size);
                }
                for (id1, id2) in &pred_skip {
                    info!("Adjust skip-bigram (pred, +step): ({}, {})", id1, id2);
                    skip_bigram_lm.adjust_cost(*id1, *id2, step_size);
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
        unigram_builder.set_unique_words(self.system_unigram_lm.unique_words);
        unigram_builder.set_total_words(self.system_unigram_lm.total_words);
        info!("Save unigram to {}", dst_unigram);
        unigram_builder.save(dst_unigram)?;
        Ok(())
    }

    pub fn save_skip_bigram(&self, dst_unigram: &str, dst_skip_bigram: &str) -> anyhow::Result<()> {
        let Some(skip_bigram_lm) = &self.system_skip_bigram_lm else {
            return Ok(());
        };

        let new_unigram = MarisaSystemUnigramLM::load(dst_unigram)?;
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        let srcmap = self.system_unigram_lm.as_hash_map();
        let src_wordid2key = srcmap
            .iter()
            .map(|(key, (word_id, _))| (*word_id, key.to_string()))
            .collect::<HashMap<i32, String>>();

        for ((word_id1, word_id2), cost) in skip_bigram_lm.as_hash_map() {
            let Some(word1) = src_wordid2key.get(&word_id1) else {
                continue;
            };
            let Some(word2) = src_wordid2key.get(&word_id2) else {
                continue;
            };
            let Some((new_word_id1, _)) = new_unigram.find(word1) else {
                continue;
            };
            let Some((new_word_id2, _)) = new_unigram.find(word2) else {
                continue;
            };
            builder.add(new_word_id1, new_word_id2, cost);
        }

        // adjustment-only エントリを追加
        for ((word_id1, word_id2), cost) in skip_bigram_lm.adjustment_only_entries() {
            let Some(word1) = src_wordid2key.get(&word_id1) else {
                continue;
            };
            let Some(word2) = src_wordid2key.get(&word_id2) else {
                continue;
            };
            let Some((new_word_id1, _)) = new_unigram.find(word1) else {
                continue;
            };
            let Some((new_word_id2, _)) = new_unigram.find(word2) else {
                continue;
            };
            builder.add(new_word_id1, new_word_id2, cost);
        }

        builder.set_default_skip_cost(skip_bigram_lm.get_default_skip_cost());
        info!("Save skip-bigram to {}", dst_skip_bigram);
        builder.save(dst_skip_bigram)?;
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

            let Some(word1) = src_wordid2key.get(&word_id1) else {
                info!("Unknown word_id: {}", word_id1);
                continue;
            };
            let Some((new_word_id1, _)) = new_unigram.find(word1) else {
                info!("Unknown word: {}", word1);
                continue;
            };

            let Some(word2) = src_wordid2key.get(&word_id2) else {
                info!("Unknown word_id: {}", word_id2);
                continue;
            };

            let Some((new_word_id2, _)) = new_unigram.find(word2) else {
                info!("Unknown word: {}", word2);
                continue;
            };
            bigram_builder.add(new_word_id1, new_word_id2, cost);
        }

        // adjustment-only エントリを追加
        for ((word_id1, word_id2), cost) in self.system_bigram_lm.adjustment_only_entries() {
            let Some(word1) = src_wordid2key.get(&word_id1) else {
                info!("Unknown word_id (adj-only): {}", word_id1);
                continue;
            };
            let Some((new_word_id1, _)) = new_unigram.find(word1) else {
                info!("Unknown word (adj-only): {}", word1);
                continue;
            };
            let Some(word2) = src_wordid2key.get(&word_id2) else {
                info!("Unknown word_id (adj-only): {}", word_id2);
                continue;
            };
            let Some((new_word_id2, _)) = new_unigram.find(word2) else {
                info!("Unknown word (adj-only): {}", word2);
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
    step_size: f32,
    may_epochs: i32,
    should_epochs: i32,
    must_epochs: i32,
    may_corpus: &str,
    should_corpus: &str,
    must_corpus: &str,
    src_unigram: &str,
    src_bigram: &str,
    src_skip_bigram: Option<&str>,
    dst_unigram: &str,
    dst_bigram: &str,
    dst_skip_bigram: Option<&str>,
) -> anyhow::Result<()> {
    let service = LearningService::new(
        src_unigram,
        src_bigram,
        src_skip_bigram,
        &[may_corpus, should_corpus, must_corpus],
    )?;

    // 実際の学習をさせる
    for (epoch, corpus) in [
        (may_epochs, may_corpus),
        (should_epochs, should_corpus),
        (must_epochs, must_corpus),
    ] {
        service.try_learn(epoch, step_size, corpus)?;
    }

    // 保存していく
    service.save_unigram(dst_unigram)?;
    service.save_bigram(dst_unigram, dst_bigram)?;
    if let Some(dst_skip) = dst_skip_bigram {
        service.save_skip_bigram(dst_unigram, dst_skip)?;
    }

    Ok(())
}
