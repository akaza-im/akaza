use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::graph::word_node::WordNode;
use anyhow::Result;
use log::{info, warn};

use crate::kana_trie::crawdad_kana_trie::CrawdadKanaTrie;
use crate::user_side_data::bigram_user_stats::BiGramUserStats;
use crate::user_side_data::unigram_user_stats::UniGramUserStats;
use crate::user_side_data::user_stats_utils::{read_user_stats_file, write_user_stats_file};

/**
 * ユーザー固有データ
 */
#[derive(Default)]
pub struct UserData {
    /// 読み仮名のトライ。入力変換時に共通接頭辞検索するために使用。
    // ここで MARISA ではなく Crawdad を採用しているのは、FFI していると std::marker::Send を実装できなくて
    // スレッドをまたいだ処理が困難になるから、以上の理由はないです。
    kana_trie: Mutex<CrawdadKanaTrie>,

    unigram_user_stats: UniGramUserStats,
    bigram_user_stats: BiGramUserStats,

    unigram_path: Option<String>,
    bigram_path: Option<String>,
    kana_trie_path: Option<String>,

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
        let kana_trie_path = basedir
            .place_data_file(Path::new("kana_trie.v1.crawdad"))?
            .to_str()
            .unwrap()
            .to_string();
        info!(
            "Load user data from default path: unigram={}, bigram={}, marisa_kana_trie={}",
            unigram_path, bigram_path, kana_trie_path
        );
        Ok(UserData::load(&unigram_path, &bigram_path, &kana_trie_path))
    }

    pub fn load(unigram_path: &String, bigram_path: &String, kana_trie_path: &String) -> Self {
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

        let kana_trie = match CrawdadKanaTrie::load(kana_trie_path) {
            Ok(trie) => trie,
            Err(err) => {
                warn!("Cannot load kana trie: {} {}", kana_trie_path, err);
                CrawdadKanaTrie::default()
            }
        };

        UserData {
            unigram_user_stats,
            bigram_user_stats,
            kana_trie: Mutex::new(kana_trie),
            unigram_path: Some(unigram_path.clone()),
            bigram_path: Some(bigram_path.clone()),
            kana_trie_path: Some(kana_trie_path.clone()),
            need_save: false,
        }
    }

    /// 入力確定した漢字のリストをユーザー統計データとして記録する。
    /// "Surface/Kana" のフォーマットで渡すこと。
    pub fn record_entries(&mut self, kanji_kanas: &Vec<String>) {
        self.unigram_user_stats.record_entries(kanji_kanas);
        self.bigram_user_stats.record_entries(kanji_kanas);
    }

    pub fn write_user_stats_file(&self) -> Result<()> {
        info!("Saving user stats file");
        if let Some(unigram_path) = &self.unigram_path {
            write_user_stats_file(unigram_path, &self.unigram_user_stats.word_count)?;
        }
        if let Some(bigram_path) = &self.bigram_path {
            write_user_stats_file(bigram_path, &self.bigram_user_stats.word_count)?;
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
