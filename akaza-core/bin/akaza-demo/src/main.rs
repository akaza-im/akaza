use std::env;
use std::fs::File;
use std::io::Write;


use libakaza::akaza_builder::AkazaBuilder;
use log::info;










fn dump_dot(fname: &str, dot: &str) {
    info!("Writing {}", fname);
    let mut file = File::create(fname).unwrap();
    file.write_all(dot.as_bytes()).unwrap();
    file.sync_all().unwrap();
}

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

    let result = akaza.convert_to_string(yomi.as_str())?;

    // dot -Tpng -o /tmp/lattice.png /tmp/lattice.dot && open /tmp/lattice.png
    // dump_dot(
    //     "/tmp/lattice-position.dot",
    //     lattice.dump_position_dot().as_str(),
    // );
    info!("RESULT IS!!! '{}'", result);
    Ok(())
}
