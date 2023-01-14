use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::graph::graph_resolver::Candidate;
use anyhow::Result;
use log::{debug, info, warn};

use crate::graph::word_node::WordNode;
use crate::kana_trie::base::KanaTrie;
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
        info!(
            "Load user data from default path: unigram={}, bigram={}",
            unigram_path, bigram_path
        );
        Ok(UserData::load(&unigram_path, &bigram_path))
    }

    pub fn load(unigram_path: &String, bigram_path: &String) -> Self {
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
        let yomis = unigram_user_stats
            .word_count
            .keys()
            .filter_map(|it| it.split_once('/'))
            .map(|(_, yomi)| yomi.to_string())
            .collect::<Vec<_>>();
        let yomi_len = yomis.len();
        let kana_trie = CedarwoodKanaTrie::build(yomis);
        let t2 = SystemTime::now();
        info!(
            "Built kana trie in {}msec({} entries)",
            t2.duration_since(t1).unwrap().as_millis(),
            yomi_len
        );
        // TODO remove this
        debug!("{:?}", kana_trie.common_prefix_search("あぐりげーしょん"));

        UserData {
            unigram_user_stats,
            bigram_user_stats,
            kana_trie: Arc::new(Mutex::new(kana_trie)),
            unigram_path: Some(unigram_path.clone()),
            bigram_path: Some(bigram_path.clone()),
            need_save: false,
        }
    }

    /// 入力確定した漢字のリストをユーザー統計データとして記録する。
    /// "Surface/Kana" のフォーマットで渡すこと。
    pub fn record_entries(&mut self, candidates: &[Candidate]) {
        self.unigram_user_stats.record_entries(candidates);
        self.bigram_user_stats.record_entries(candidates);

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

    pub fn write_user_stats_file(&mut self) -> Result<()> {
        if self.need_save {
            info!("Saving user stats file");
            if let Some(unigram_path) = &self.unigram_path {
                write_user_stats_file(unigram_path, &self.unigram_user_stats.word_count)?;
            }
            if let Some(bigram_path) = &self.bigram_path {
                write_user_stats_file(bigram_path, &self.bigram_user_stats.word_count)?;
            }

            self.need_save = false;
        }

        Ok(())
    }

    pub fn get_unigram_cost(&self, kanji: &str, yomi: &str) -> Option<f32> {
        self.unigram_user_stats
            .get_cost(format!("{}/{}", kanji, yomi))
    }

    pub fn get_bigram_cost(&self, node1: &WordNode, node2: &WordNode) -> Option<f32> {
        self.bigram_user_stats
            .get_cost(node1.key().as_str(), node2.key().as_str())
    }
}
