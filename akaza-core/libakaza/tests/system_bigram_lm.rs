#[cfg(test)]
mod tests {
    use anyhow::Context;

    use libakaza::lm::system_bigram::SystemBigramLM;
    use libakaza::lm::system_unigram_lm::SystemUnigramLM;

    fn basedir() -> String {
        env!("CARGO_MANIFEST_DIR").to_string()
    }

    fn datadir() -> String {
        basedir() + "/../../akaza-data/data/"
    }

    fn load_unigram() -> SystemUnigramLM {
        let datadir = datadir();
        let path = datadir + "/lm_v2_1gram.trie";
        SystemUnigramLM::load(&path).unwrap()
    }

    fn load_bigram() -> SystemBigramLM {
        let datadir = datadir();
        let path = datadir + "/lm_v2_2gram.trie";
        let bigram_lm = SystemBigramLM::load(&path).unwrap();
        bigram_lm
    }

    #[test]
    fn test_load() {
        let unigram: SystemUnigramLM = load_unigram();
        let bigram = load_bigram();

        let (id1, score1) = unigram.find("私/わたし").unwrap();
        assert!(id1 > 0);
        assert!(score1 < 0.0_f32);

        let (id2, score2) = unigram.find("から/から").unwrap();
        assert!(id2 > 0);
        assert!(score2 < 0.0_f32);

        println!("id1={}, id2={}", id1, id2);

        let bigram_score = bigram
            .get_edge_cost(id1, id2)
            .with_context(|| {
                format!(
                    "bigram.num_entries={} id1={} id2={}",
                    bigram.num_keys(),
                    id1,
                    id2
                )
            })
            .unwrap();
        assert!(bigram_score < 0.0_f32);

        println!("BigramScore={}", bigram_score)
    }
}
