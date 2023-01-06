#[cfg(test)]
mod tests {
    use libakaza::lm::system_unigram_lm::SystemUnigramLM;

    fn basedir() -> String {
        env!("CARGO_MANIFEST_DIR").to_string()
    }

    fn datadir() -> String {
        basedir() + "/../akaza-data/data/"
    }

    #[test]
    fn test_load() {
        let path = datadir() + "/lm_v2_1gram.trie";
        let lm = SystemUnigramLM::load(&path).unwrap();
        let (id, score) = lm.find("私/わたし").unwrap();
        assert!(id > 0);
        assert!(score > 0.0_f32);

        println!("Score={}", score)
    }
}
