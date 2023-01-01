use std::env;

use libakaza::akaza_builder::AkazaBuilder;

// fn dump_dot(fname: &str, dot: &str) {
//     info!("Writing {}", fname);
//     let mut file = File::create(fname).unwrap();
//     file.write_all(dot.as_bytes()).unwrap();
//     file.sync_all().unwrap();
// }

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args: Vec<String> = env::args().collect();
    let datadir = args[1].to_owned();
    let yomi = args[2].to_owned();

    let akaza = AkazaBuilder::default()
        .system_data_dir(datadir.as_str())
        .build()?;

    let result = akaza.convert(yomi.as_str())?;
    for terms in result {
        println!("- {}/{}({})", terms[0].kanji, terms[0].yomi, terms[0].cost);
        let words: Vec<String> = terms
            .iter()
            .skip(1)
            .map(|f| format!("{}/{}({})", f.kanji, f.yomi, f.cost))
            .collect();
        println!("    {}", words.join(","));
    }

    // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
    // dump_dot(
    //     "/tmp/lattice-position.dot",
    //     lattice.dump_position_dot().as_str(),
    // );
    // info!("RESULT IS!!! '{}'", result);
    Ok(())
}
