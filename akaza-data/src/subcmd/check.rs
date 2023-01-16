use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use encoding_rs::{EUC_JP, UTF_8};
use log::info;

use libakaza::dict::merge_dict::merge_dict;
use libakaza::dict::skk::read::read_skkdict;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;
use libakaza::user_side_data::user_data::UserData;

pub fn check(yomi: &str, expected: Option<String>, user_data: bool) -> anyhow::Result<()> {
    let dict = merge_dict(vec![
        read_skkdict(Path::new("skk-dev-dict/SKK-JISYO.L"), EUC_JP)?,
        read_skkdict(Path::new("data/SKK-JISYO.akaza"), UTF_8)?,
    ]);

    let mut builder = BigramWordViterbiEngineBuilder::new(Some(dict), None);
    if user_data {
        info!("Enabled user data");
        let user_data = UserData::load_from_default_path()?;
        builder.user_data(Arc::new(Mutex::new(user_data)));
        builder.load_user_config(true);
    }
    let engine = builder.build()?;
    let lattice = engine.to_lattice(yomi, None)?;
    if let Some(expected) = expected {
        let _dot = lattice.dump_cost_dot(expected.as_str());
        println!("{}", _dot);
        let mut file = File::create("/tmp/dump.dot")?;
        file.write_all(_dot.as_bytes())?;
    }

    let got = engine.resolve(&lattice)?;
    let terms: Vec<String> = got.iter().map(|f| f[0].kanji.clone()).collect();
    let result = terms.join("/");
    println!("{}", result);

    Ok(())
}
