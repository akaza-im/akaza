use libakaza::akaza_builder::AkazaBuilder;

pub fn check(yomi: &str) -> anyhow::Result<()> {
    let datadir = env!("CARGO_MANIFEST_DIR").to_string() + "/data/";

    let akaza = AkazaBuilder::default().system_data_dir(&datadir).build()?;
    let lattice = akaza.to_lattice(yomi, &Vec::new())?;
    println!("{}", lattice.dump_cost_dot());
    println!("{:?}", lattice);

    let got = akaza.resolve(&lattice)?;
    let terms: Vec<String> = got.iter().map(|f| f[0].kanji.clone()).collect();
    let result = terms.join("");
    println!("{}", result);

    Ok(())
}
