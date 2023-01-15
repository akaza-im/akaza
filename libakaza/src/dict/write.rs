use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use log::info;

use crate::skk::merge_skkdict::merge_skkdict;

pub fn write_skk_dict(
    ofname: &str,
    dicts: Vec<HashMap<String, Vec<String>>>,
) -> anyhow::Result<()> {
    info!("Writing {}", ofname);
    let merged_dict = merge_skkdict(dicts);
    {
        let mut wfp = File::create(ofname)?;
        wfp.write_all(";; okuri-ari entries.\n".as_bytes())?;
        wfp.write_all(";; okuri-nasi entries.\n".as_bytes())?;
        for (yomi, kanjis) in merged_dict.iter() {
            assert!(!yomi.is_empty(), "yomi must not be empty: {:?}", kanjis);
            let kanjis = kanjis.join("/");
            wfp.write_fmt(format_args!("{} /{}/\n", yomi, kanjis))?;
        }
    }
    Ok(())
}
