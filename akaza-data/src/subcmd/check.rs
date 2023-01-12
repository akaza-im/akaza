use libakaza::engine::base::HenkanEngine;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;
use std::fs::File;
use std::io::Write;

pub fn check(yomi: &str, expected: Option<String>) -> anyhow::Result<()> {
    let datadir = env!("CARGO_MANIFEST_DIR").to_string() + "/data/";

    let akaza = BigramWordViterbiEngineBuilder::new(&datadir).build()?;
    let lattice = akaza.to_lattice(yomi, None)?;
    if let Some(expected) = expected {
        let _dot = lattice.dump_cost_dot(expected.as_str());
        println!("{}", _dot);
        let mut file = File::create("/tmp/dump.dot")?;
        file.write_all(_dot.as_bytes())?;
    }

    let got = akaza.resolve(&lattice)?;
    let terms: Vec<String> = got.iter().map(|f| f[0].kanji.clone()).collect();
    let result = terms.join("/");
    println!("{}", result);

    Ok(())
}
