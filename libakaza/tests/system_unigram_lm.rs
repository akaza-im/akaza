#[cfg(test)]
#[cfg(feature = "it")]
mod tests {
    use libakaza::lm::base::SystemUnigramLM;
    use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;

    fn basedir() -> String {
        env!("CARGO_MANIFEST_DIR").to_string()
    }

    fn datadir() -> String {
        basedir() + "/../akaza-data/data/"
    }

    #[test]
    fn test_load() {
        let path = datadir() + "/stats-vibrato-unigram.trie";
        let lm = MarisaSystemUnigramLM::load(&path).unwrap();
        let (id, score) = lm.find("私/わたし").unwrap();
        assert!(id > 0);
        assert!(score > 0.0_f32);

        println!("Score={}", score)
    }
}
