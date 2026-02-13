use anyhow::Result;
use log::info;
use rustc_hash::FxHashMap;

use libakaza::cost::calc_cost;
use libakaza::lm::base::SystemUnigramLM;
use libakaza::lm::system_skip_bigram::MarisaSystemSkipBigramLMBuilder;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;

use crate::wordcnt::wordcnt_skip_bigram::WordcntSkipBigram;
use crate::wordcnt::wordcnt_unigram::WordcntUnigram;

/// wordcnt skip-bigram trie (カウントベース) → skip_bigram.model (f16 コストベース) に変換する。
///
/// wordcnt trie 内の word_id は wordcnt unigram trie 由来のため、
/// 最終モデル (unigram.model) の word_id へ再マッピングする必要がある。
pub fn convert_skip_bigram_model(
    src_skip_bigram: &str,
    src_wordcnt_unigram: &str,
    dst_unigram_model: &str,
    dst: &str,
) -> Result<()> {
    // 1. wordcnt unigram trie をロードして、旧 word_id → 単語キー のマッピングを作る
    info!("Loading wordcnt unigram: {}", src_wordcnt_unigram);
    let wordcnt_unigram = WordcntUnigram::load(src_wordcnt_unigram)?;
    let old_map = wordcnt_unigram.to_count_hashmap(); // HashMap<String, (old_word_id, count)>
    let old_id_to_word: FxHashMap<i32, String> = old_map
        .iter()
        .map(|(word, (id, _))| (*id, word.clone()))
        .collect();
    info!("  old unigram entries: {}", old_id_to_word.len());

    // 2. unigram.model をロードして、単語キー → 新 word_id のマッピングを作る
    info!("Loading unigram model: {}", dst_unigram_model);
    let new_unigram = MarisaSystemUnigramLM::load(dst_unigram_model)?;

    // 3. wordcnt skip-bigram trie をロード
    info!("Loading wordcnt skip-bigram: {}", src_skip_bigram);
    let wordcnt = WordcntSkipBigram::load(src_skip_bigram)?;
    let cnt_map = wordcnt.to_cnt_map();

    info!(
        "total_words={}, unique_words={}, entries={}",
        wordcnt.total_words,
        wordcnt.unique_words,
        cnt_map.len()
    );

    // 4. 旧 word_id → 新 word_id にマッピングしながらモデルを構築
    let mut builder = MarisaSystemSkipBigramLMBuilder::default();
    let mut mapped = 0_usize;
    let mut skipped = 0_usize;

    for ((old_id1, old_id2), cnt) in &cnt_map {
        // 旧ID → 単語キー
        let (Some(word1), Some(word2)) = (old_id_to_word.get(old_id1), old_id_to_word.get(old_id2))
        else {
            skipped += 1;
            continue;
        };

        // 単語キー → 新ID
        let (Some((new_id1, _)), Some((new_id2, _))) =
            (new_unigram.find(word1), new_unigram.find(word2))
        else {
            skipped += 1;
            continue;
        };

        let cost = calc_cost(*cnt, wordcnt.total_words, wordcnt.unique_words);
        builder.add(new_id1, new_id2, cost);
        mapped += 1;
    }

    info!("Mapped {} entries, skipped {} entries", mapped, skipped);

    // デフォルトコスト = カウント0 のペアのコスト（最大ペナルティ）
    let default_cost = calc_cost(0, wordcnt.total_words, wordcnt.unique_words);
    info!("Default skip cost (count=0): {}", default_cost);
    builder.set_default_skip_cost(default_cost);

    info!("Saving skip-bigram model: {}", dst);
    builder.save(dst)?;

    info!("DONE");
    Ok(())
}
