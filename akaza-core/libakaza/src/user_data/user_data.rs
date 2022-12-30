use log::warn;
use std::collections::HashMap;
use std::io;

use crate::kana_trie::KanaTrie;
use crate::user_data::bigram_user_stats::BiGramUserStats;
use crate::user_data::unigram_user_stats::UniGramUserStats;
use crate::user_data::user_stats_utils::{read_user_stats_file, write_user_stats_file};
use marisa_sys::Marisa;

/**
 * ユーザー固有データ
 */
struct UserData {
    /// 読み仮名のトライ。入力変換時に共通接頭辞検索するために使用。
    kana_trie: KanaTrie,
    unigram_user_stats: UniGramUserStats,
    bigram_user_stats: BiGramUserStats,

    unigram_path: String,
    bigram_path: String,
    kana_trie_path: String,

    pub need_save: bool,
}
impl UserData {
    fn load(unigram_path: &String, bigram_path: &String, kana_trie_path: &String) -> UserData {
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

        let kana_trie = match KanaTrie::load(kana_trie_path) {
            Ok(trie) => trie,
            Err(err) => {
                warn!("Cannot load kana trie: {} {}", kana_trie_path, err);
                KanaTrie::new(Marisa::new())
            }
        };

        UserData {
            unigram_user_stats,
            bigram_user_stats,
            kana_trie,
            unigram_path: unigram_path.clone(),
            bigram_path: bigram_path.clone(),
            kana_trie_path: kana_trie_path.clone(),
            need_save: false,
        }
    }

    /// 入力確定した漢字のリストをユーザー統計データとして記録する。
    fn record_entries(&mut self, kanjis: Vec<String>, _kanas: Vec<String>) {
        self.unigram_user_stats.record_entries(&kanjis);
        self.bigram_user_stats.record_entries(&kanjis);

        // for kana in kanas {
        // TODO: record kanas to trie.
        // }
    }

    fn write_user_stats_file(&self) -> Result<(), io::Error> {
        write_user_stats_file(&self.unigram_path, &self.unigram_user_stats.word_count)?;
        write_user_stats_file(&self.bigram_path, &self.bigram_user_stats.word_count)?;
        Ok(())
    }
}
