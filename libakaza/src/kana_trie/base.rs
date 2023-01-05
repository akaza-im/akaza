pub trait KanaTrie {
    fn common_prefix_search(&self, query: &str) -> Vec<String>;
}
