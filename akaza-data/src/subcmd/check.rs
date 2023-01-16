use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

use log::info;

use libakaza::config::{Config, DictConfig};
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;
use libakaza::user_side_data::user_data::UserData;

pub fn check(yomi: &str, expected: Option<String>, user_data: bool) -> anyhow::Result<()> {
    let mut builder = BigramWordViterbiEngineBuilder::new(Config {
        dicts: vec![
            DictConfig {
                dict_type: "skk".to_string(),
                encoding: Some("euc-jp".to_string()),
                path: "skk-dev-dict/SKK-JISYO.L".to_string(),
            },
            DictConfig {
                dict_type: "skk".to_string(),
                encoding: Some("utf-8".to_string()),
                path: "data/SKK-JISYO.akaza".to_string(),
            },
        ],
        single_term: None,
    });
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
