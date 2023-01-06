use libakaza::akaza_builder::AkazaBuilder;

pub fn check(yomi: &String) -> anyhow::Result<()> {
    let datadir = env!("CARGO_MANIFEST_DIR").to_string() + "/data/";

    let akaza = AkazaBuilder::default().system_data_dir(&datadir).build()?;
    let p = akaza.to_lattice(yomi, &Vec::new())?;
    println!("{}", p.dump_cost_dot());
    println!("{:?}", p);

    Ok(())
}
