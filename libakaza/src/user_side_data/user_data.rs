use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::Result;
use encoding_rs::UTF_8;
use log::{info, warn};
use rustc_hash::FxHashMap;

use crate::dict::skk::read::read_skkdict;
use crate::dict::skk::write::write_skk_dict;
use crate::graph::candidate::Candidate;
use crate::graph::word_node::WordNode;
use crate::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use crate::user_side_data::bigram_user_stats::BiGramUserStats;
use crate::user_side_data::skip_bigram_user_stats::SkipBigramUserStats;
use crate::user_side_data::unigram_user_stats::UniGramUserStats;
use crate::user_side_data::user_stats_utils::{
    read_user_dict_v2, read_user_stats_file, read_user_stats_file_v2, write_user_dict_v2,
    write_user_stats_file, write_user_stats_file_v2,
};

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
    skip_bigram_user_stats: SkipBigramUserStats,

    unigram_path: Option<String>,
    bigram_path: Option<String>,
    skip_bigram_path: Option<String>,
    dict_path: Option<String>,

    // v2 暗号化ファイルパス
    unigram_v2_path: Option<String>,
    bigram_v2_path: Option<String>,
    skip_bigram_v2_path: Option<String>,
    dict_v2_path: Option<String>,

    encryption_key: Option<Vec<u8>>,

    pub dict: FxHashMap<String, Vec<String>>,

    pub(crate) need_save: bool,
}

/// v1 または v2 からユーザー統計データを読み込むヘルパー。
/// v2 ファイルが存在すれば v2 を優先し、なければ v1 にフォールバックする。
fn load_user_stats(
    v1_path: &str,
    v2_path: &str,
    key: Option<&[u8]>,
    label: &str,
) -> Vec<(String, u32)> {
    // v2 ファイルがあり、鍵もあれば v2 を試す
    if let Some(key) = key {
        if Path::new(v2_path).exists() {
            match read_user_stats_file_v2(v2_path, key) {
                Ok(dat) => return dat,
                Err(err) => {
                    warn!("Cannot load v2 {} data from {}: {}", label, v2_path, err);
                }
            }
        }
    }
    // v1 にフォールバック
    match read_user_stats_file(&v1_path.to_string()) {
        Ok(dat) => dat,
        Err(err) => {
            warn!("Cannot load {} data from {}: {}", label, v1_path, err);
            Vec::new()
        }
    }
}

/// v2 → v1 (compound_dict.v1.txt) → 旧 SKK-JISYO.user の順でフォールバック
fn load_user_dict(
    dict_path: &str,
    dict_v2_path: &str,
    key: Option<&[u8]>,
) -> FxHashMap<String, Vec<String>> {
    // v2 暗号化ファイルを試す
    if let Some(key) = key {
        if Path::new(dict_v2_path).exists() {
            match read_user_dict_v2(dict_v2_path, key) {
                Ok(dict) => return dict,
                Err(err) => {
                    warn!("Cannot load v2 dict data from {}: {}", dict_v2_path, err);
                }
            }
        }
    }
    // v1 (compound_dict.v1.txt) にフォールバック
    if Path::new(dict_path).exists() {
        match read_skkdict(Path::new(dict_path), UTF_8) {
            Ok(d) => return d.into_iter().collect(),
            Err(err) => {
                warn!("Cannot load user dict: {:?} {:?}", dict_path, err);
            }
        }
    }
    // 旧 SKK-JISYO.user からのマイグレーション
    if let Some(parent) = Path::new(dict_path).parent() {
        let legacy_path = parent.join("SKK-JISYO.user");
        if legacy_path.exists() {
            info!(
                "Migrating dict from legacy SKK-JISYO.user: {:?}",
                legacy_path
            );
            match read_skkdict(&legacy_path, UTF_8) {
                Ok(d) => return d.into_iter().collect(),
                Err(err) => {
                    warn!("Cannot load legacy user dict: {:?} {:?}", legacy_path, err);
                }
            }
        }
    }
    Default::default()
}

fn build_unigram_stats(dat: Vec<(String, u32)>) -> UniGramUserStats {
    let unique_count = dat.len() as u32;
    let total_count: u32 = dat.iter().map(|f| f.1).sum();
    let mut word_count: FxHashMap<String, u32> = FxHashMap::default();
    for (word, count) in dat {
        word_count.insert(word, count);
    }
    UniGramUserStats::new(unique_count, total_count, word_count)
}

fn build_bigram_stats(dat: Vec<(String, u32)>) -> BiGramUserStats {
    let unique_count = dat.len() as u32;
    let total_count: u32 = dat.iter().map(|f| f.1).sum();
    let mut words_count: FxHashMap<String, u32> = FxHashMap::default();
    for (words, count) in dat {
        words_count.insert(words, count);
    }
    BiGramUserStats::new(unique_count, total_count, words_count)
}

fn build_skip_bigram_stats(dat: Vec<(String, u32)>) -> SkipBigramUserStats {
    let unique_count = dat.len() as u32;
    let total_count: u32 = dat.iter().map(|f| f.1).sum();
    let mut words_count: FxHashMap<String, u32> = FxHashMap::default();
    for (words, count) in dat {
        words_count.insert(words, count);
    }
    SkipBigramUserStats::new(unique_count, total_count, words_count)
}

impl UserData {
    pub fn load_from_default_path(key: Option<&[u8]>) -> Result<Self> {
        let basedir = crate::xdg_dirs::BaseDirectories::with_prefix("akaza")?;
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
        let skip_bigram_path = basedir
            .place_data_file(Path::new("skip_bigram.v1.txt"))?
            .to_str()
            .unwrap()
            .to_string();
        let dict_path = basedir
            .place_data_file(Path::new("compound_dict.v1.txt"))?
            .to_str()
            .unwrap()
            .to_string();

        let unigram_v2_path = basedir
            .place_data_file(Path::new("unigram.v2.bin"))?
            .to_str()
            .unwrap()
            .to_string();
        let bigram_v2_path = basedir
            .place_data_file(Path::new("bigram.v2.bin"))?
            .to_str()
            .unwrap()
            .to_string();
        let skip_bigram_v2_path = basedir
            .place_data_file(Path::new("skip_bigram.v2.bin"))?
            .to_str()
            .unwrap()
            .to_string();
        let dict_v2_path = basedir
            .place_data_file(Path::new("compound_dict.v2.bin"))?
            .to_str()
            .unwrap()
            .to_string();

        info!(
            "Load user data from default path: unigram={}, bigram={}, skip_bigram={}",
            unigram_path, bigram_path, skip_bigram_path
        );
        Ok(UserData::load(
            &unigram_path,
            &bigram_path,
            &skip_bigram_path,
            &dict_path,
            &unigram_v2_path,
            &bigram_v2_path,
            &skip_bigram_v2_path,
            &dict_v2_path,
            key,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load(
        unigram_path: &str,
        bigram_path: &str,
        skip_bigram_path: &str,
        dict_path: &str,
        unigram_v2_path: &str,
        bigram_v2_path: &str,
        skip_bigram_v2_path: &str,
        dict_v2_path: &str,
        key: Option<&[u8]>,
    ) -> Self {
        // ユーザーデータが読み込めないことは fatal エラーではない。
        // 初回起動時にはデータがないので。
        // データがなければ初期状態から始める
        let unigram_dat = load_user_stats(unigram_path, unigram_v2_path, key, "unigram");
        let unigram_user_stats = build_unigram_stats(unigram_dat);

        let bigram_dat = load_user_stats(bigram_path, bigram_v2_path, key, "bigram");
        let bigram_user_stats = build_bigram_stats(bigram_dat);

        let skip_bigram_dat =
            load_user_stats(skip_bigram_path, skip_bigram_v2_path, key, "skip-bigram");
        let skip_bigram_user_stats = build_skip_bigram_stats(skip_bigram_dat);

        let dict: FxHashMap<String, Vec<String>> = load_user_dict(dict_path, dict_v2_path, key);

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
            skip_bigram_user_stats,
            dict,
            kana_trie: Arc::new(Mutex::new(kana_trie)),
            unigram_path: Some(unigram_path.to_string()),
            bigram_path: Some(bigram_path.to_string()),
            skip_bigram_path: Some(skip_bigram_path.to_string()),
            dict_path: Some(dict_path.to_string()),
            unigram_v2_path: Some(unigram_v2_path.to_string()),
            bigram_v2_path: Some(bigram_v2_path.to_string()),
            skip_bigram_v2_path: Some(skip_bigram_v2_path.to_string()),
            dict_v2_path: Some(dict_v2_path.to_string()),
            encryption_key: key.map(|k| k.to_vec()),
            need_save: false,
        }
    }

    /// 入力確定した漢字のリストをユーザー統計データとして記録する。
    /// "Surface/Kana" のフォーマットで渡すこと。
    pub fn record_entries(&mut self, candidates: &[Candidate]) {
        self.unigram_user_stats.record_entries(candidates);
        self.bigram_user_stats.record_entries(candidates);
        self.skip_bigram_user_stats.record_entries(candidates);

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
                "Saving user stats file: unigram={:?},{}, bigram={:?},{}, skip_bigram={:?},{}",
                self.unigram_path,
                self.unigram_user_stats.word_count.len(),
                self.bigram_path,
                self.bigram_user_stats.word_count.len(),
                self.skip_bigram_path,
                self.skip_bigram_user_stats.word_count.len(),
            );

            if let Some(key) = &self.encryption_key {
                // v2 暗号化形式で保存
                if let Some(path) = &self.unigram_v2_path {
                    write_user_stats_file_v2(path, key, &self.unigram_user_stats.word_count)?;
                }
                if let Some(path) = &self.bigram_v2_path {
                    write_user_stats_file_v2(path, key, &self.bigram_user_stats.word_count)?;
                }
                if let Some(path) = &self.skip_bigram_v2_path {
                    write_user_stats_file_v2(path, key, &self.skip_bigram_user_stats.word_count)?;
                }
            } else {
                // v1 テキスト形式で保存（鍵なしの場合）
                if let Some(unigram_path) = &self.unigram_path {
                    write_user_stats_file(unigram_path, &self.unigram_user_stats.word_count)?;
                }
                if let Some(bigram_path) = &self.bigram_path {
                    write_user_stats_file(bigram_path, &self.bigram_user_stats.word_count)?;
                }
                if let Some(skip_bigram_path) = &self.skip_bigram_path {
                    write_user_stats_file(
                        skip_bigram_path,
                        &self.skip_bigram_user_stats.word_count,
                    )?;
                }
            }

            if let Some(key) = &self.encryption_key {
                if let Some(path) = &self.dict_v2_path {
                    write_user_dict_v2(path, key, &self.dict)?;
                }
            } else if let Some(dict_path) = &self.dict_path {
                write_skk_dict(dict_path, vec![self.dict.clone().into_iter().collect()])?;
            }

            self.need_save = false;
        }

        Ok(())
    }

    pub fn get_unigram_cost(&self, node: &WordNode) -> Option<f32> {
        self.unigram_user_stats.get_cost(&node.key())
    }

    pub fn get_bigram_cost(&self, node1: &WordNode, node2: &WordNode) -> Option<f32> {
        self.bigram_user_stats
            .get_cost(node1.key().as_str(), node2.key().as_str())
    }

    pub fn get_skip_bigram_cost(&self, node1: &WordNode, node2: &WordNode) -> Option<f32> {
        self.skip_bigram_user_stats
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

    #[test]
    fn test_record_entries_records_bos_eos_bigram() {
        let mut user_data = UserData::default();
        // 「互換」を確定
        user_data.record_entries(&[Candidate::new("ごかん", "互換", 0.0)]);

        // BOS→互換 のユーザー bigram が記録されていること
        let bos_node = WordNode::create_bos();
        let gokan_node = WordNode::new(0, "互換", "ごかん", None, false);
        let bos_cost = user_data.get_bigram_cost(&bos_node, &gokan_node);
        assert!(
            bos_cost.is_some(),
            "BOS→互換 のユーザー bigram が記録されていない"
        );

        // 互換→EOS のユーザー bigram も記録されていること
        let eos_node = WordNode::create_eos(6);
        let eos_cost = user_data.get_bigram_cost(&gokan_node, &eos_node);
        assert!(
            eos_cost.is_some(),
            "互換→EOS のユーザー bigram が記録されていない"
        );
    }

    /// ユーザーが「互換」を多く選択した場合、「五感」よりBOS bigram コストが低くなること
    #[test]
    fn test_bos_bigram_reflects_user_preference() {
        let mut user_data = UserData::default();
        // 「互換」を5回、「五感」を2回確定
        for _ in 0..5 {
            user_data.record_entries(&[Candidate::new("ごかん", "互換", 0.0)]);
        }
        for _ in 0..2 {
            user_data.record_entries(&[Candidate::new("ごかん", "五感", 0.0)]);
        }

        let bos_node = WordNode::create_bos();
        let gokan_node = WordNode::new(0, "互換", "ごかん", None, false);
        let gokan2_node = WordNode::new(0, "五感", "ごかん", None, false);

        let cost_gokan = user_data.get_bigram_cost(&bos_node, &gokan_node).unwrap();
        let cost_gokan2 = user_data.get_bigram_cost(&bos_node, &gokan2_node).unwrap();

        assert!(
            cost_gokan < cost_gokan2,
            "BOS→互換({}) のコストが BOS→五感({}) より低くなるべき",
            cost_gokan,
            cost_gokan2
        );
    }

    #[test]
    fn test_counter_learning_generalizes_across_numbers() {
        let mut user_data = UserData::default();
        user_data.record_entries(&[Candidate::new("3しゅうかん", "3週間", 0.0)]);

        let learned =
            user_data.get_unigram_cost(&WordNode::new(0, "3週間", "3しゅうかん", None, false));
        assert!(learned.is_some());

        let generalized =
            user_data.get_unigram_cost(&WordNode::new(0, "516週間", "516しゅうかん", None, false));
        assert!(generalized.is_some());
    }

    #[test]
    fn test_v1_to_v2_migration() {
        use crate::user_side_data::user_stats_utils::write_user_stats_file;
        use tempfile::TempDir;

        let tmpdir = TempDir::new().unwrap();
        let dir = tmpdir.path();

        // v1 ファイルを作成
        let unigram_v1 = dir.join("unigram.v1.txt").to_str().unwrap().to_string();
        let bigram_v1 = dir.join("bigram.v1.txt").to_str().unwrap().to_string();
        let skip_bigram_v1 = dir.join("skip_bigram.v1.txt").to_str().unwrap().to_string();
        let dict_path = dir
            .join("compound_dict.v1.txt")
            .to_str()
            .unwrap()
            .to_string();

        let unigram_v2 = dir.join("unigram.v2.bin").to_str().unwrap().to_string();
        let bigram_v2 = dir.join("bigram.v2.bin").to_str().unwrap().to_string();
        let skip_bigram_v2 = dir.join("skip_bigram.v2.bin").to_str().unwrap().to_string();
        let dict_v2 = dir
            .join("compound_dict.v2.bin")
            .to_str()
            .unwrap()
            .to_string();

        let mut wc: FxHashMap<String, u32> = FxHashMap::default();
        wc.insert("渡し/わたし".to_string(), 3);
        write_user_stats_file(&unigram_v1, &wc).unwrap();
        write_user_stats_file(&bigram_v1, &FxHashMap::default()).unwrap();
        write_user_stats_file(&skip_bigram_v1, &FxHashMap::default()).unwrap();

        let key = [0x42u8; 32];

        // v2 ファイルがないので v1 からロードされる
        let mut user_data = UserData::load(
            &unigram_v1,
            &bigram_v1,
            &skip_bigram_v1,
            &dict_path,
            &unigram_v2,
            &bigram_v2,
            &skip_bigram_v2,
            &dict_v2,
            Some(&key),
        );

        // 保存すると v2 形式で書き出される
        user_data.need_save = true;
        user_data.write_user_files().unwrap();

        // v2 ファイルが作成されていること
        assert!(
            Path::new(&unigram_v2).exists(),
            "v2 unigram file should exist"
        );

        // v2 から再読み込みできること
        let user_data2 = UserData::load(
            &unigram_v1,
            &bigram_v1,
            &skip_bigram_v1,
            &dict_path,
            &unigram_v2,
            &bigram_v2,
            &skip_bigram_v2,
            &dict_v2,
            Some(&key),
        );
        let cost = user_data2.get_unigram_cost(&WordNode::new(0, "渡し", "わたし", None, false));
        assert!(
            cost.is_some(),
            "v2 から読み込んだデータでコストが取得できるべき"
        );
    }

    #[test]
    fn test_compound_dict_v2_roundtrip() {
        use tempfile::TempDir;

        let tmpdir = TempDir::new().unwrap();
        let dir = tmpdir.path();

        let unigram_v1 = dir.join("unigram.v1.txt").to_str().unwrap().to_string();
        let bigram_v1 = dir.join("bigram.v1.txt").to_str().unwrap().to_string();
        let skip_bigram_v1 = dir.join("skip_bigram.v1.txt").to_str().unwrap().to_string();
        let dict_path = dir
            .join("compound_dict.v1.txt")
            .to_str()
            .unwrap()
            .to_string();
        let unigram_v2 = dir.join("unigram.v2.bin").to_str().unwrap().to_string();
        let bigram_v2 = dir.join("bigram.v2.bin").to_str().unwrap().to_string();
        let skip_bigram_v2 = dir.join("skip_bigram.v2.bin").to_str().unwrap().to_string();
        let dict_v2 = dir
            .join("compound_dict.v2.bin")
            .to_str()
            .unwrap()
            .to_string();

        let key = [0x42u8; 32];

        // compound_word を含むデータを記録して保存
        let mut user_data = UserData::load(
            &unigram_v1,
            &bigram_v1,
            &skip_bigram_v1,
            &dict_path,
            &unigram_v2,
            &bigram_v2,
            &skip_bigram_v2,
            &dict_v2,
            Some(&key),
        );

        // compound_word を直接追加
        user_data
            .dict
            .entry("ごじょうほう".to_string())
            .or_default()
            .push("ご情報".to_string());
        user_data.need_save = true;
        user_data.write_user_files().unwrap();

        // compound_dict.v2.bin が作成されていること
        assert!(
            Path::new(&dict_v2).exists(),
            "compound_dict.v2.bin should exist"
        );

        // v2 から再読み込みして dict が復元されること
        let user_data2 = UserData::load(
            &unigram_v1,
            &bigram_v1,
            &skip_bigram_v1,
            &dict_path,
            &unigram_v2,
            &bigram_v2,
            &skip_bigram_v2,
            &dict_v2,
            Some(&key),
        );
        assert_eq!(
            user_data2.dict.get("ごじょうほう"),
            Some(&vec!["ご情報".to_string()]),
            "compound_dict が v2 から正しく復元されるべき"
        );
    }

    #[test]
    fn test_legacy_skk_jisyo_user_migration() {
        use crate::dict::skk::write::write_skk_dict;
        use tempfile::TempDir;

        let tmpdir = TempDir::new().unwrap();
        let dir = tmpdir.path();

        // 旧 SKK-JISYO.user にデータを書き込み
        let legacy_path = dir.join("SKK-JISYO.user");
        let mut legacy_dict: FxHashMap<String, Vec<String>> = FxHashMap::default();
        legacy_dict.insert("ごじょうほう".to_string(), vec!["ご情報".to_string()]);
        write_skk_dict(
            legacy_path.to_str().unwrap(),
            vec![legacy_dict.into_iter().collect()],
        )
        .unwrap();

        // compound_dict.v1.txt は存在しない状態で load
        let dict_path = dir
            .join("compound_dict.v1.txt")
            .to_str()
            .unwrap()
            .to_string();
        let dict_v2 = dir
            .join("compound_dict.v2.bin")
            .to_str()
            .unwrap()
            .to_string();
        let unigram_v1 = dir.join("unigram.v1.txt").to_str().unwrap().to_string();
        let bigram_v1 = dir.join("bigram.v1.txt").to_str().unwrap().to_string();
        let skip_bigram_v1 = dir.join("skip_bigram.v1.txt").to_str().unwrap().to_string();
        let unigram_v2 = dir.join("unigram.v2.bin").to_str().unwrap().to_string();
        let bigram_v2 = dir.join("bigram.v2.bin").to_str().unwrap().to_string();
        let skip_bigram_v2 = dir.join("skip_bigram.v2.bin").to_str().unwrap().to_string();

        let user_data = UserData::load(
            &unigram_v1,
            &bigram_v1,
            &skip_bigram_v1,
            &dict_path,
            &unigram_v2,
            &bigram_v2,
            &skip_bigram_v2,
            &dict_v2,
            None,
        );

        // 旧 SKK-JISYO.user からマイグレーションされていること
        assert_eq!(
            user_data.dict.get("ごじょうほう"),
            Some(&vec!["ご情報".to_string()]),
            "旧 SKK-JISYO.user からマイグレーションされるべき"
        );
    }
}
