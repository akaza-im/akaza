use libakaza::lm::system_unigram_lm::SystemUnigramLM;

pub fn dump_unigram_dict(filename: &str) -> anyhow::Result<()> {
    let dict = SystemUnigramLM::load(filename)?;
    let id_map = dict.as_id_map();
    for yomi in id_map.keys() {
        let (word_id, score) = dict.find(yomi.as_str()).unwrap();
        println!("{} {} {}", yomi, word_id, score);
    }

    Ok(())
}
