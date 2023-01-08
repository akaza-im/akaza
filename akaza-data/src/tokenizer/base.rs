pub trait AkazaTokenizer {
    fn tokenize(&self, src: &str) -> anyhow::Result<String>;
}
