use libakaza::lm::base::SystemUnigramLM;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;

pub fn dump_unigram_dict(filename: &str) -> anyhow::Result<()> {
    let dict = MarisaSystemUnigramLM::load(filename)?;
    let dict_map = dict.as_hash_map();
    for yomi in dict_map.keys() {
        let (word_id, score) = dict.find(yomi.as_str()).unwrap();
        println!("{} {} {}", yomi, word_id, score);
    }

    Ok(())
}
