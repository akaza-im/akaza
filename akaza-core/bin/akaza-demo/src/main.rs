use std::env;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use log::info;

use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::kana_kanji_dict::KanaKanjiDict;
use libakaza::kana_trie::KanaTrieBuilder;
use libakaza::lm::system_bigram::SystemBigramLM;
use libakaza::lm::system_unigram_lm::SystemUnigramLM;
use libakaza::user_side_data::user_data::UserData;

fn dump_dot(fname: &str, dot: &str) {
    info!("Writing {}", fname);
    let mut file = File::create(fname).unwrap();
    file.write_all(dot.as_bytes()).unwrap();
    file.sync_all().unwrap();
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args: Vec<String> = env::args().collect();
    let datadir = args[1].to_owned();
    let yomi = args[2].to_owned();

    let system_unigram_path = &(datadir.to_string() + "/lm_v2_1gram.trie");
    let system_unigram_lm = SystemUnigramLM::load(system_unigram_path).unwrap();
    info!(
        "system-unigram-lm: {} entries",
        system_unigram_lm.num_keys()
    );
    let system_bigram_path = &(datadir.to_string() + "/lm_v2_2gram.trie");
    let system_bigram_lm = SystemBigramLM::load(system_bigram_path).unwrap();
    info!("system-bgram-lm: {} entries", system_bigram_lm.num_keys());

    let system_kana_kanji_dict = KanaKanjiDict::load(&(datadir + "/system_dict.trie")).unwrap();
    let mut system_dict_yomis_builder = KanaTrieBuilder::default();
    for yomi in system_kana_kanji_dict.all_yomis().unwrap() {
        system_dict_yomis_builder.add(&yomi);
    }
    let system_kana_trie = system_dict_yomis_builder.build();

    let graph_builder = Segmenter::new(vec![system_kana_trie]);
    let segmentation_result = graph_builder.build("わたし");
    dump_dot(
        "/tmp/segmentation-result.dot",
        segmentation_result.dump_dot().as_str(),
    );

    let user_data = UserData::default();

    let graph_builder = GraphBuilder::new(
        system_kana_kanji_dict,
        Rc::new(user_data),
        Rc::new(system_unigram_lm),
        Rc::new(system_bigram_lm),
    );
    let lattice = graph_builder.construct(&yomi, segmentation_result);
    // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
    dump_dot(
        "/tmp/lattice-position.dot",
        lattice.dump_position_dot().as_str(),
    );
    dump_dot("/tmp/lattice-cost.dot", lattice.dump_cost_dot().as_str());
    let resolver = GraphResolver::default();
    let result = resolver.viterbi(&yomi, lattice);
    info!("RESULT IS!!! '{}'", result.unwrap());
}
