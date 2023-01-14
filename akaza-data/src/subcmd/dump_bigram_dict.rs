use libakaza::lm::base::{SystemBigramLM, SystemUnigramLM};
use libakaza::lm::system_bigram::MarisaSystemBigramLM;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;
use std::collections::HashMap;

pub fn dump_bigram_dict(unigram_file: &str, bigram_file: &str) -> anyhow::Result<()> {
    let unigram = MarisaSystemUnigramLM::load(unigram_file)?;
    let unigram_map = unigram
        .as_hash_map()
        .iter()
        .map(|(key, (id, _))| (*id, key.to_string()))
        .collect::<HashMap<i32, String>>();

    let bigram = MarisaSystemBigramLM::load(bigram_file)?;
    for ((word_id1, word_id2), cost) in bigram.as_hash_map() {
        let key1 = unigram_map.get(&word_id1).unwrap();
        let key2 = unigram_map.get(&word_id2).unwrap();
        println!("{} {} {}", cost, key1, key2);
    }

    Ok(())
}
