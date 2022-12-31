#[cfg(test)]
mod tests {
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
        let unigram_lm = SystemUnigramLM::load(&path).unwrap();
        return unigram_lm
    }

    fn load_bigram() -> SystemUnigramLM {
        let datadir = datadir();
        let path = datadir + "/lm_v2_2gram.trie";
        let unigram_lm = SystemBigramLM::load(&path).unwrap();
        return unigram_lm
    }

    #[test]
    fn test_load() {
        let unigram = load_unigram();
        let unigram = load_bigram();

        let (id, score) = lm.find("私/わたし").unwrap();
        assert!(id > 0);
        assert!(score < 0.0_f32);

        println!("Score={}", score)
    }
}
