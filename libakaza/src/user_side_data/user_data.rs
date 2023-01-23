use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::Result;
use encoding_rs::UTF_8;
use log::{info, warn};

use crate::dict::skk::read::read_skkdict;
use crate::dict::skk::write::write_skk_dict;
use crate::graph::candidate::Candidate;
use crate::graph::word_node::WordNode;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::user_side_data::bigram_user_stats::BiGramUserStats;
use crate::user_side_data::unigram_user_stats::UniGramUserStats;
use crate::user_side_data::user_stats_utils::{read_user_stats_file, write_user_stats_file};

/**
 * ユーザー固有データ
 */
#[derive(Default)]
pub struct UserData {
    /// 読み仮名のトライ。入力変換時に共通接頭辞検索するために使用。
    // ここで MARISA ではなく Cedarwood を採用しているのは
    // - FFI していると std::marker::Send を実装できなくてスレッドをまたいだ処理が困難になるから
    // - 更新可能なトライ構造だから
    pub(crate) kana_trie: Arc<Mutex<CedarwoodKanaTrie>>,

    unigram_user_stats: UniGramUserStats,
    bigram_user_stats: BiGramUserStats,

    unigram_path: Option<String>,
    bigram_path: Option<String>,
    dict_path: Option<String>,

    pub dict: HashMap<String, Vec<String>>,

    pub(crate) need_save: bool,
}

impl UserData {
    pub fn load_from_default_path() -> Result<Self> {
        let basedir = xdg::BaseDirectories::with_prefix("akaza")?;
        let unigram_path = basedir
            .place_data_file(Path::new("unigram.v1.txt"))?
            .to_str()
            .unwrap()
            .to_string();
        let bigram_path = basedir
            .place_data_file(Path::new("bigram.v1.txt"))?
            .to_str()
            .unwrap()
            .to_string();
        let dict_path = basedir
            .place_data_file(Path::new("SKK-JISYO.user"))?
            .to_str()
            .unwrap()
            .to_string();
        info!(
            "Load user data from default path: unigram={}, bigram={}",
            unigram_path, bigram_path
        );
        Ok(UserData::load(&unigram_path, &bigram_path, &dict_path))
    }

    pub fn load(unigram_path: &String, bigram_path: &String, dict_path: &String) -> Self {
        // ユーザーデータが読み込めないことは fatal エラーではない。
        // 初回起動時にはデータがないので。
        // データがなければ初期所状態から始める
        let unigram_user_stats = match read_user_stats_file(unigram_path) {
            Ok(dat) => {
                let unique_count = dat.len() as u32;
                let total_count: u32 = dat.iter().map(|f| f.1).sum();
                let mut word_count: HashMap<String, u32> = HashMap::new();
                for (word, count) in dat {
                    word_count.insert(word, count);
                }
                UniGramUserStats::new(unique_count, total_count, word_count)
            }
            Err(err) => {
                warn!(
                    "Cannot load user unigram data from {}: {}",
                    unigram_path, err
                );

                UniGramUserStats::new(0, 0, HashMap::new())
            }
        };

        // build bigram
        let bigram_user_stats = match read_user_stats_file(bigram_path) {
            Ok(dat) => {
                let unique_count = dat.len() as u32;
                let total_count: u32 = dat.iter().map(|f| f.1).sum();
                let mut words_count: HashMap<String, u32> = HashMap::new();
                for (words, count) in dat {
                    words_count.insert(words, count);
                }
                BiGramUserStats::new(unique_count, total_count, words_count)
            }
            Err(err) => {
                warn!("Cannot load user bigram data from {}: {}", bigram_path, err);
                // ユーザーデータは初回起動時などにはないので、データがないものとして処理を続行する
                BiGramUserStats::new(0, 0, HashMap::new())
            }
        };

        let dict = match read_skkdict(Path::new(dict_path), UTF_8) {
            Ok(d) => d,
            Err(err) => {
                warn!("Cannot load user dict: {:?} {:?}", dict_path, err);
                Default::default()
            }
        };

        // let kana_trie = match CedarwoodKanaTrie::load(kana_trie_path) {
        //     Ok(trie) => trie,
        //     Err(err) => {
        //         warn!("Cannot load kana trie: {} {}", kana_trie_path, err);
        //         CedarwoodKanaTrie::default()
        //     }
        // };

        // cedarwood トライを構築する。
        // キャッシュせずに動的に構築する方向性。
        let t1 = SystemTime::now();
        let mut yomis = unigram_user_stats
            .word_count
            .keys()
            .filter_map(|it| it.split_once('/'))
            .map(|(_, yomi)| yomi.to_string())
            .collect::<Vec<_>>();
        // ユーザー辞書の内容も追加
        dict.keys().for_each(|yomi| yomis.push(yomi.to_string()));
        let yomi_len = yomis.len();
        let kana_trie = CedarwoodKanaTrie::build(yomis);
        let t2 = SystemTime::now();
        info!(
            "Built kana trie in {}msec({} entries)",
            t2.duration_since(t1).unwrap().as_millis(),
            yomi_len
        );

        UserData {
            unigram_user_stats,
            bigram_user_stats,
            dict,
            kana_trie: Arc::new(Mutex::new(kana_trie)),
            unigram_path: Some(unigram_path.clone()),
            bigram_path: Some(bigram_path.clone()),
            dict_path: Some(dict_path.clone()),
            need_save: false,
        }
    }

    /// 入力確定した漢字のリストをユーザー統計データとして記録する。
    /// "Surface/Kana" のフォーマットで渡すこと。
    pub fn record_entries(&mut self, candidates: &[Candidate]) {
        self.unigram_user_stats.record_entries(candidates);
        self.bigram_user_stats.record_entries(candidates);

        // 複合語として覚えておくべきものがあれば、学習する。
        candidates
            .iter()
            .filter(|candidate| candidate.compound_word)
            .for_each(|candidate| {
                self.dict
                    .entry(candidate.yomi.to_string())
                    .or_default()
                    .push(candidate.surface.to_string())
            });

        // かなトライを更新する
        let mut kana_trie = self.kana_trie.lock().unwrap();
        candidates
            .iter()
            .map(|it| it.yomi.to_string())
            .for_each(|it| {
                if !kana_trie.contains(it.as_str()) {
                    kana_trie.update(it.as_str())
                }
            });

        self.need_save = true;
    }

    pub fn write_user_files(&mut self) -> Result<()> {
        if self.need_save {
            info!(
                "Saving user stats file: unigram={:?},{}, bigram={:?},{}",
                self.unigram_path,
                self.unigram_user_stats.word_count.len(),
                self.bigram_path,
                self.bigram_user_stats.word_count.len(),
            );
            if let Some(unigram_path) = &self.unigram_path {
                write_user_stats_file(unigram_path, &self.unigram_user_stats.word_count)?;
            }
            if let Some(bigram_path) = &self.bigram_path {
                write_user_stats_file(bigram_path, &self.bigram_user_stats.word_count)?;
            }
            if let Some(dict_path) = &self.dict_path {
                write_skk_dict(dict_path, vec![self.dict.clone()])?;
            }

            self.need_save = false;
        }

        Ok(())
    }

    pub fn get_unigram_cost(&self, node: &WordNode) -> Option<f32> {
        self.unigram_user_stats.get_cost(node.key())
    }

    pub fn get_bigram_cost(&self, node1: &WordNode, node2: &WordNode) -> Option<f32> {
        self.bigram_user_stats
            .get_cost(node1.key().as_str(), node2.key().as_str())
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::*;

    #[test]
    fn test_record_entries() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .try_init();

        let mut user_data = UserData::default();
        let cost1 = user_data.get_unigram_cost(&WordNode::new(
            0,
            "アグリゲーション",
            "あぐりげーしょん",
            None,
            false,
        ));
        assert_eq!(cost1, None);
        user_data.record_entries(&[Candidate::new(
            "あぐりげーしょん",
            "アグリゲーション",
            0_f32,
        )]);
        let cost2 = user_data
            .get_unigram_cost(&WordNode::new(
                0,
                "アグリゲーション",
                "あぐりげーしょん",
                None,
                false,
            ))
            .unwrap();
        user_data.record_entries(&[Candidate::new(
            "あぐりげーしょん",
            "アグリゲーション",
            0_f32,
        )]);
        let cost3 = user_data
            .get_unigram_cost(&WordNode::new(
                0,
                "アグリゲーション",
                "あぐりげーしょん",
                None,
                false,
            ))
            .unwrap();
        info!("{}, {}", cost2, cost3);
        assert!(cost2 > cost3);
    }
}
