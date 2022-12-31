#[cfg(test)]
mod tests {

    use libakaza::akaza_builder::AkazaBuilder;

    #[test]
    fn test_wnn() -> anyhow::Result<()> {
        let datadir = env!("CARGO_MANIFEST_DIR").to_string() + "/../../akaza-data/data/";
        let akaza = AkazaBuilder::default()
            .system_data_dir(datadir.as_str())
            .build()?;

        let got = akaza.convert_to_string("わたしのなまえはなかのです")?;
        assert_eq!(got, "私の名前は中野です");
        Ok(())
    }
}
