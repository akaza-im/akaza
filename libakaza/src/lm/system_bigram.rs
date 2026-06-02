use std::cell::RefCell;
use std::fs::File;
use std::io::{BufWriter, Read as _, Write as _};

use rustc_hash::FxHashMap;

use anyhow::{bail, Result};
use half::f16;
use log::{info, warn};

use rsmarisa::{Agent, Keyset, Trie};

use crate::lm::base::SystemBigramLM;
use crate::lm::model_metadata::{add_metadata_to_keyset, read_metadata_from_trie, ModelMetadata};

/*
   trie key:
   {word1 ID}    # 3 bytes
   {word2 ID}    # 3 bytes

   scores file (separate):
   [4B] num_entries (u32 LE)
   [2B × num_entries] f16 scores (LE)
*/

const DEFAULT_COST_KEY: &str = "__DEFAULT_EDGE_COST__";

/**
 * bigram 言語モデル。
 * unigram の生成のときに得られた単語IDを利用することで、圧縮している。
 */
pub struct MarisaSystemBigramLMBuilder {
    keyset: Keyset,
    metadata: ModelMetadata,
    entries: Vec<(i32, i32, f32)>,
}

impl Default for MarisaSystemBigramLMBuilder {
    fn default() -> Self {
        Self {
            keyset: Keyset::new(),
            metadata: ModelMetadata::default(),
            entries: Vec::new(),
        }
    }
}

impl MarisaSystemBigramLMBuilder {
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

    pub fn set_default_edge_cost(&mut self, score: f32) -> &mut Self {
        let key = format!("{DEFAULT_COST_KEY}\t{score}");
        self.keyset.push_back_str(&key).unwrap();
        self
    }

    pub fn set_metadata(&mut self, metadata: ModelMetadata) -> &mut Self {
        self.metadata = metadata;
        self
    }

    fn build_scores(trie: &Trie, entries: &[(i32, i32, f32)]) -> Vec<f16> {
        // エントリの (id1, id2) → score マップを構築
        let mut entry_map: FxHashMap<(i32, i32), f32> = FxHashMap::default();
        for &(id1, id2, score) in entries {
            entry_map.insert((id1, id2), score);
        }

        // trie の全キーを走査して key_id 順に scores を構築
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

    pub fn build(&mut self) -> Result<MarisaSystemBigramLM> {
        add_metadata_to_keyset(&mut self.keyset, &self.metadata);
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        let default_edge_cost = MarisaSystemBigramLM::read_default_edge_cost(&trie)?;
        let scores = Self::build_scores(&trie, &self.entries);
        Ok(MarisaSystemBigramLM {
            trie,
            default_edge_cost,
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

pub struct MarisaSystemBigramLM {
    trie: Trie,
    default_edge_cost: f32,
    scores: Vec<f16>,
    /// 検索用 Agent の再利用（毎回のアロケーションを避ける）
    agent: RefCell<Agent>,
}

impl MarisaSystemBigramLM {
    pub fn load(filename: &str) -> Result<MarisaSystemBigramLM> {
        info!("Loading system-bigram: {}", filename);
        let mut trie = Trie::new();
        trie.load(filename)?;
        let default_edge_cost = Self::read_default_edge_cost(&trie)?;

        // scores ファイルを読み込む
        let scores_path = format!("{filename}.scores");
        let scores = Self::load_scores(&scores_path)?;
        info!("Loaded {} score entries from {}", scores.len(), scores_path);

        Ok(MarisaSystemBigramLM {
            trie,
            default_edge_cost,
            scores,
            agent: RefCell::new(Agent::new()),
        })
    }

    fn load_scores(path: &str) -> Result<Vec<f16>> {
        // 旧実装は 2 バイトずつ read_exact + push する loop だった。
        // bigram で 18M, skip_bigram で 30M entry あるので syscall/branch のオーバーヘッドが
        // 起動時間にそのまま乗っていた。Vec<f16> を必要長で確保し、その underlying bytes
        // に対して 1 回の read_exact で埋める形に変更。
        let mut file = File::open(path)?;
        let mut buf4 = [0u8; 4];
        file.read_exact(&mut buf4)?;
        let num_entries = u32::from_le_bytes(buf4) as usize;

        let mut scores: Vec<f16> = vec![f16::ZERO; num_entries];
        // SAFETY: f16 は `#[repr(transparent)]` over u16 の Copy 型なので任意ビット列が
        // 有効。ファイルフォーマットは LE 2 バイト/entry で、x86_64 はリトルエンディアン
        // のため、underlying バイト列にそのまま read_exact できる。
        // 他 arch (big endian) でも、半精度の生ビット列を読み込むだけなので未定義動作
        // にはならない (値の解釈は別問題だが、現状 akaza は LE プラットフォームのみ想定)。
        let bytes_len = num_entries * std::mem::size_of::<f16>();
        if bytes_len > 0 {
            let bytes: &mut [u8] = unsafe {
                std::slice::from_raw_parts_mut(scores.as_mut_ptr() as *mut u8, bytes_len)
            };
            file.read_exact(bytes)?;
        }
        Ok(scores)
    }

    pub fn num_keys(&self) -> usize {
        self.trie.num_keys()
    }

    /// bigram に (word_id1, word_id2) のエントリが含まれているか検索する。
    /// @return Some(score) if found, None otherwise.
    pub fn lookup(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        self.get_edge_cost(word_id1, word_id2)
    }

    pub fn metadata(&self) -> ModelMetadata {
        read_metadata_from_trie(&self.trie)
    }

    fn read_default_edge_cost(trie: &Trie) -> Result<f32> {
        let mut agent = Agent::new();
        agent.set_query_str(DEFAULT_COST_KEY);

        if trie.predictive_search(&mut agent) {
            let key = agent.key().as_str();
            if let Some((_, score)) = key.split_once('\t') {
                return Ok(score.parse::<f32>()?);
            }
        }

        bail!("Cannot read default cost from bigram-trie");
    }
}

impl SystemBigramLM for MarisaSystemBigramLM {
    fn get_default_edge_cost(&self) -> f32 {
        self.default_edge_cost
    }

    /**
     * edge cost を得る。
     * この ID は、unigram の trie でふられたもの。
     */
    fn get_edge_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
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
            } else {
                warn!(
                    "Bigram key_id {} out of bounds (scores len={})",
                    key_id,
                    self.scores.len()
                );
            }
        }

        None
    }

    fn as_hash_map(&self) -> FxHashMap<(i32, i32), f32> {
        let mut map: FxHashMap<(i32, i32), f32> = FxHashMap::default();
        let mut agent = Agent::new();
        agent.set_query_str("");

        while self.trie.predictive_search(&mut agent) {
            let word = agent.key().as_bytes();
            let key_id = agent.key().id();
            if word.len() == 6 {
                let word_id1 = i32::from_le_bytes([word[0], word[1], word[2], 0]);
                let word_id2 = i32::from_le_bytes([word[3], word[4], word[5], 0]);
                if key_id < self.scores.len() {
                    let cost = self.scores[key_id].to_f32();
                    map.insert((word_id1, word_id2), cost);
                }
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn build_and_load() -> anyhow::Result<()> {
        let mut builder = MarisaSystemBigramLMBuilder::default();
        builder.set_default_edge_cost(20_f32);
        builder.add(4649, 5963, 5.11_f32);
        let system_bigram_lm = builder.build()?;
        let got_score = system_bigram_lm.get_edge_cost(4649, 5963).unwrap();
        assert!(5.0 < got_score && got_score < 5.12);

        let map = system_bigram_lm.as_hash_map();
        assert!(map.contains_key(&(4649, 5963)));
        let g = *map.get(&(4649, 5963)).unwrap();
        assert!(5.10_f32 < g && g < 5.12_f32);

        Ok(())
    }

    #[test]
    fn save_and_load_roundtrip() -> anyhow::Result<()> {
        let mut builder = MarisaSystemBigramLMBuilder::default();
        builder.set_default_edge_cost(14.3);
        builder.add(100, 200, 3.5);
        builder.add(100, 300, 7.25);
        builder.add(500, 600, 1.0);

        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap().to_string();
        builder.save(&path)?;

        let lm = MarisaSystemBigramLM::load(&path)?;

        // default edge cost
        assert!((lm.get_default_edge_cost() - 14.3).abs() < 0.01);

        // lookup
        let c1 = lm.get_edge_cost(100, 200).unwrap();
        assert!(3.4 < c1 && c1 < 3.6, "got {c1}");

        let c2 = lm.get_edge_cost(100, 300).unwrap();
        assert!(7.2 < c2 && c2 < 7.3, "got {c2}");

        let c3 = lm.get_edge_cost(500, 600).unwrap();
        assert!(0.9 < c3 && c3 < 1.1, "got {c3}");

        // miss
        assert!(lm.get_edge_cost(999, 888).is_none());

        // as_hash_map
        let map = lm.as_hash_map();
        assert_eq!(map.len(), 3);
        assert!(map.contains_key(&(100, 200)));

        // cleanup scores file
        let _ = std::fs::remove_file(format!("{path}.scores"));

        Ok(())
    }
}
