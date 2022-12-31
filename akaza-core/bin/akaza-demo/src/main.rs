use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::kana_kanji_dict::KanaKanjiDictBuilder;
use libakaza::kana_trie::KanaTrie;
use libakaza::lm::system_unigram_lm::SystemUnigramLM;
use libakaza::user_side_data::user_data::UserData;
use std::env;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use tempfile::NamedTempFile;

fn main() {
    let _ = env_logger::builder().try_init();

    let args: Vec<String> = env::args().collect();
    let datadir = args[1].to_owned();
    let system_unigram_path = &(datadir.to_string() + "/lm_v2_1gram.trie");
    let system_unigram_lm = SystemUnigramLM::load(system_unigram_path).unwrap();

    let system_dict = KanaTrie::load(&(datadir + "/system_dict.trie")).unwrap();

    let graph_builder = Segmenter::new(vec![system_dict]);
    let graph = graph_builder.build("わたし");

    let mut dict_builder = KanaKanjiDictBuilder::default();
    dict_builder.add("わたし", "私/渡し");

    let yomi = "わたし".to_string();

    // TODO このへん、ちょっとコピペしまくらないといけなくて渋い。
    let dict = dict_builder.build();
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
