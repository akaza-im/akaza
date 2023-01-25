use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

use log::info;

use libakaza::config::{DictConfig, DictEncoding, DictType, DictUsage, EngineConfig};
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;
use libakaza::user_side_data::user_data::UserData;

pub fn check(
    yomi: &str,
    expected: Option<String>,
    user_data: bool,
    eucjp_dict: &Vec<String>,
    utf8_dict: &Vec<String>,
    model_dir: &str,
) -> anyhow::Result<()> {
    let mut dicts: Vec<DictConfig> = Vec::new();
    for path in eucjp_dict {
        dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::EucJp,
            path: path.clone(),
            usage: DictUsage::Normal,
        })
    }

    for path in utf8_dict {
        dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::Utf8,
            path: path.clone(),
            usage: DictUsage::Normal,
        })
    }

    let mut builder = BigramWordViterbiEngineBuilder::new(EngineConfig {
        dicts,
        model: model_dir.to_string(),
        dict_cache: false,
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
    let terms: Vec<String> = got.iter().map(|f| f[0].surface.clone()).collect();
    let result = terms.join("/");
    println!("{}", result);

    Ok(())
}
