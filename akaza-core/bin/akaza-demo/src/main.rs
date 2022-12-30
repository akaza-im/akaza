use libakaza::graph_builder::{GraphBuilder, GraphResolver, Segmenter};
use libakaza::kana_kanji_dict::KanaKanjiDictBuilder;
use libakaza::kana_trie::KanaTrieBuilder;
use libakaza::lm::system_unigram_lm::{SystemUnigramLM, SystemUnigramLMBuilder};
use libakaza::user_side_data::user_data::UserData;
use std::env;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use tempfile::NamedTempFile;

fn main() {
    let _ = env_logger::builder().try_init();

    let args: Vec<String> = env::args().collect();
    let datadir = &args[1];
    let system_unigram_lm =
        SystemUnigramLM::load(&(datadir.to_owned() + "/lm_v2_1gram.trie")).unwrap();

    let mut builder = KanaTrieBuilder::new();
    builder.add(&"わたし".to_string());
    builder.add(&"わた".to_string());
    builder.add(&"し".to_string());
    let kana_trie = builder.build();

    let graph_builder = Segmenter::new(vec![kana_trie]);
    let graph = graph_builder.build(&"わたし".to_string());

    let mut dict_builder = KanaKanjiDictBuilder::default();
    dict_builder.add("わたし", "私/渡し");

    let yomi = "わたし".to_string();

    // TODO このへん、ちょっとコピペしまくらないといけなくて渋い。
    let dict = dict_builder.build();
    let system_unigram_lm_builder = SystemUnigramLMBuilder::default();
    let mut user_data = UserData::load(
        &NamedTempFile::new()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string(),
        &NamedTempFile::new()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string(),
        &NamedTempFile::new()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string(),
    );
    // 私/わたし のスコアをガッと上げる。
    user_data.record_entries(vec!["私/わたし".to_string()]);
    let graph_builder = GraphBuilder::new(dict, Rc::new(user_data), Rc::new(system_unigram_lm));
    let lattice = graph_builder.construct(&yomi, graph);
    // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
    File::create("/tmp/lattice.dot")
        .unwrap()
        .write_all(lattice.dump_dot().as_bytes())
        .unwrap();
    let resolver = GraphResolver::new();
    let result = resolver.viterbi(&yomi, lattice);
    assert_eq!(result, "私");

    println!("Hello, world!");
}
