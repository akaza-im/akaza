use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::kana_kanji_dict::{KanaKanjiDict, KanaKanjiDictBuilder};
use libakaza::kana_trie::KanaTrieBuilder;
use libakaza::lm::system_unigram_lm::SystemUnigramLM;
use libakaza::user_side_data::user_data::UserData;
use log::info;
use std::env;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use tempfile::NamedTempFile;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args: Vec<String> = env::args().collect();
    let datadir = args[1].to_owned();
    let yomi = args[2].to_owned();
    let system_unigram_path = &(datadir.to_string() + "/lm_v2_1gram.trie");
    let system_unigram_lm = SystemUnigramLM::load(system_unigram_path).unwrap();

    let system_kana_kanji_dict = KanaKanjiDict::load(&(datadir + "/system_dict.trie")).unwrap();
    let mut system_dict_yomis_builder = KanaTrieBuilder::default();
    for yomi in system_kana_kanji_dict.all_yomis().unwrap() {
        system_dict_yomis_builder.add(&yomi);
    }
    let system_kana_trie = system_dict_yomis_builder.build();

    let graph_builder = Segmenter::new(vec![system_kana_trie]);
    let graph = graph_builder.build("わたし");

    let mut dict_builder = KanaKanjiDictBuilder::default();
    dict_builder.add("わたし", "私/渡し");

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
    let resolver = GraphResolver::default();
    let result = resolver.viterbi(&yomi, lattice);
    info!("RESULT IS!!! '{}'", result);
}
