use std::cell::RefCell;
use std::fs::File;
use std::io::{BufWriter, Read as _, Write as _};

use anyhow::{bail, Result};
use half::f16;
use log::info;
use rustc_hash::FxHashMap;

use rsmarisa::{Agent, Keyset, Trie};

use crate::lm::base::SystemSkipBigramLM;
use crate::lm::model_metadata::{add_metadata_to_keyset, read_metadata_from_trie, ModelMetadata};

/*
   trie key:
   {word1 ID}    # 3 bytes (w_{i-2})
   {word2 ID}    # 3 bytes (w_i)

   scores file (separate):
   [4B] num_entries (u32 LE)
   [2B × num_entries] f16 scores (LE)
*/

const DEFAULT_COST_KEY: &str = "__DEFAULT_SKIP_COST__";

/// skip-bigram 言語モデルのビルダー。
/// trie キーは `[3B id1][3B id2]`、スコアは別ファイル。
pub struct MarisaSystemSkipBigramLMBuilder {
    keyset: Keyset,
    metadata: ModelMetadata,
    entries: Vec<(i32, i32, f32)>,
}

impl Default for MarisaSystemSkipBigramLMBuilder {
    fn default() -> Self {
        Self {
            keyset: Keyset::new(),
            metadata: ModelMetadata::default(),
            entries: Vec::new(),
        }
    }
}

impl MarisaSystemSkipBigramLMBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, score: f32) {
        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();

        assert_eq!(id1_bytes[3], 0);
        assert_eq!(id2_bytes[3], 0);

        let key: [u8; 6] = [
            id1_bytes[0],
            id1_bytes[1],
            id1_bytes[2],
            id2_bytes[0],
            id2_bytes[1],
            id2_bytes[2],
        ];
        self.keyset.push_back_bytes(&key, 1.0).unwrap();
        self.entries.push((word_id1, word_id2, score));
    }

    pub fn set_default_skip_cost(&mut self, cost: f32) -> &mut Self {
        let key = format!("{DEFAULT_COST_KEY}\t{cost}");
        self.keyset.push_back_str(&key).unwrap();
        self
    }

    pub fn set_metadata(&mut self, metadata: ModelMetadata) -> &mut Self {
        self.metadata = metadata;
        self
    }

    fn build_scores(trie: &Trie, entries: &[(i32, i32, f32)]) -> Vec<f16> {
        let mut entry_map: FxHashMap<(i32, i32), f32> = FxHashMap::default();
        for &(id1, id2, score) in entries {
            entry_map.insert((id1, id2), score);
        }

        let num_keys = trie.num_keys();
        let mut scores = vec![f16::ZERO; num_keys];

        let mut agent = Agent::new();
        agent.set_query_str("");
        while trie.predictive_search(&mut agent) {
            let key_bytes = agent.key().as_bytes();
            let key_id = agent.key().id();
            if key_bytes.len() == 6 {
                let word_id1 = i32::from_le_bytes([key_bytes[0], key_bytes[1], key_bytes[2], 0]);
                let word_id2 = i32::from_le_bytes([key_bytes[3], key_bytes[4], key_bytes[5], 0]);
                if let Some(&score) = entry_map.get(&(word_id1, word_id2)) {
                    scores[key_id] = f16::from_f32(score);
                }
            }
        }

        scores
    }

    pub fn build(&mut self) -> Result<MarisaSystemSkipBigramLM> {
        add_metadata_to_keyset(&mut self.keyset, &self.metadata);
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        let default_skip_cost = MarisaSystemSkipBigramLM::read_default_skip_cost(&trie)?;
        let scores = Self::build_scores(&trie, &self.entries);
        Ok(MarisaSystemSkipBigramLM {
            trie,
            default_skip_cost,
            scores,
            agent: RefCell::new(Agent::new()),
        })
    }

    pub fn save(&mut self, ofname: &str) -> Result<()> {
        add_metadata_to_keyset(&mut self.keyset, &self.metadata);
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        trie.save(ofname)?;

        // scores ファイルを書き出す
        let scores = Self::build_scores(&trie, &self.entries);
        let scores_path = format!("{ofname}.scores");
        let mut writer = BufWriter::new(File::create(&scores_path)?);
        let num_entries = scores.len() as u32;
        writer.write_all(&num_entries.to_le_bytes())?;
        for s in &scores {
            writer.write_all(&s.to_le_bytes())?;
        }
        writer.flush()?;
        info!("Saved {} score entries to {}", num_entries, scores_path);

        Ok(())
    }
}

pub struct MarisaSystemSkipBigramLM {
    trie: Trie,
    default_skip_cost: f32,
    scores: Vec<f16>,
    /// 検索用 Agent の再利用（毎回のアロケーションを避ける）
    agent: RefCell<Agent>,
}

impl MarisaSystemSkipBigramLM {
    pub fn metadata(&self) -> ModelMetadata {
        read_metadata_from_trie(&self.trie)
    }

    pub fn load(filename: &str) -> Result<MarisaSystemSkipBigramLM> {
        info!("Loading system-skip-bigram: {}", filename);
        let mut trie = Trie::new();
        trie.load(filename)?;
        let default_skip_cost = Self::read_default_skip_cost(&trie).unwrap_or_else(|_| {
            info!("No default skip cost in model, using fallback 10.0");
            10.0
        });
        info!("  default_skip_cost={}", default_skip_cost);

        // scores ファイルを読み込む
        let scores_path = format!("{filename}.scores");
        let scores = Self::load_scores(&scores_path)?;
        info!("Loaded {} score entries from {}", scores.len(), scores_path);

        Ok(MarisaSystemSkipBigramLM {
            trie,
            default_skip_cost,
            scores,
            agent: RefCell::new(Agent::new()),
        })
    }

    fn load_scores(path: &str) -> Result<Vec<f16>> {
        // 旧実装は 2 バイトずつ read_exact + push する loop だった (system_bigram と同様)。
        // skip_bigram は 30M entry あり、起動時間に大きく寄与していたため一括 read に変更。
        let mut file = File::open(path)?;
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let num_entries = u32::from_le_bytes(buf4) as usize;

        let mut scores: Vec<f16> = vec![f16::ZERO; num_entries];
        // SAFETY: f16 は `#[repr(transparent)]` over u16 の Copy 型なので任意ビット列が
        // 有効。ファイルフォーマットは LE 2 バイト/entry で、x86_64 はリトルエンディアン
        // のため、underlying バイト列にそのまま read_exact できる。
        let bytes_len = num_entries * std::mem::size_of::<f16>();
        if bytes_len > 0 {
            let bytes: &mut [u8] = unsafe {
                std::slice::from_raw_parts_mut(scores.as_mut_ptr() as *mut u8, bytes_len)
            };
            file.read_exact(bytes)?;
        }
        Ok(scores)
    }

    fn read_default_skip_cost(trie: &Trie) -> Result<f32> {
        let mut agent = Agent::new();
        agent.set_query_str(DEFAULT_COST_KEY);

        if trie.predictive_search(&mut agent) {
            let key = agent.key().as_str();
            if let Some((_, score)) = key.split_once('\t') {
                return Ok(score.parse::<f32>()?);
            }
        }

        bail!("Cannot read default skip cost from skip-bigram trie");
    }
}

impl SystemSkipBigramLM for MarisaSystemSkipBigramLM {
    fn get_skip_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();
        let key: [u8; 6] = [
            id1_bytes[0],
            id1_bytes[1],
            id1_bytes[2],
            id2_bytes[0],
            id2_bytes[1],
            id2_bytes[2],
        ];

        let mut agent = self.agent.borrow_mut();
        agent.set_query_bytes(&key);

        if self.trie.lookup(&mut agent) {
            let key_id = agent.key().id();
            if key_id < self.scores.len() {
                return Some(self.scores[key_id].to_f32());
            }
        }

        None
    }

    fn get_default_skip_cost(&self) -> f32 {
        self.default_skip_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn build_and_lookup() -> anyhow::Result<()> {
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        builder.add(100, 200, 3.5);
        builder.add(100, 300, 4.0);
        builder.set_default_skip_cost(10.0);
        let lm = builder.build()?;

        let cost = lm.get_skip_cost(100, 200).unwrap();
        assert!(3.4 < cost && cost < 3.6);

        let cost = lm.get_skip_cost(100, 300).unwrap();
        assert!(3.9 < cost && cost < 4.1);

        assert!(lm.get_skip_cost(999, 888).is_none());
        assert!((lm.get_default_skip_cost() - 10.0).abs() < f32::EPSILON);
        Ok(())
    }

    #[test]
    fn save_and_load_roundtrip() -> anyhow::Result<()> {
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        builder.set_default_skip_cost(10.0);
        builder.add(100, 200, 3.5);
        builder.add(500, 600, 7.0);

        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap().to_string();
        builder.save(&path)?;

        let lm = MarisaSystemSkipBigramLM::load(&path)?;

        assert!((lm.get_default_skip_cost() - 10.0).abs() < 0.01);

        let c1 = lm.get_skip_cost(100, 200).unwrap();
        assert!(3.4 < c1 && c1 < 3.6, "got {c1}");

        let c2 = lm.get_skip_cost(500, 600).unwrap();
        assert!(6.9 < c2 && c2 < 7.1, "got {c2}");

        assert!(lm.get_skip_cost(999, 888).is_none());

        // cleanup scores file
        let _ = std::fs::remove_file(format!("{path}.scores"));

        Ok(())
    }

    #[test]
    fn default_cost_fallback() -> anyhow::Result<()> {
        // デフォルトコスト未設定の古いモデル → フォールバック値 10.0
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        builder.add(1, 2, 5.0);
        // set_default_skip_cost を呼ばない
        add_metadata_to_keyset(&mut builder.keyset, &builder.metadata);
        let mut trie = Trie::new();
        trie.build(&mut builder.keyset, 0);
        let result = MarisaSystemSkipBigramLM::read_default_skip_cost(&trie);
        assert!(result.is_err());
        Ok(())
    }
}
