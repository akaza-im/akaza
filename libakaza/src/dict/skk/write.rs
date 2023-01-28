use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use log::info;

use crate::dict::merge_dict::merge_dict;

pub fn write_skk_dict(
    ofname: &str,
    dicts: Vec<HashMap<String, Vec<String>>>,
) -> anyhow::Result<()> {
    info!("Writing {}", ofname);
    let merged_dict = merge_dict(dicts);
    {
        let mut wfp = File::create(ofname)?;
        wfp.write_all(";; okuri-ari entries.\n".as_bytes())?;
        wfp.write_all(";; okuri-nasi entries.\n".as_bytes())?;
        let mut keys = merged_dict.keys().collect::<Vec<_>>();
        keys.sort();
        for yomi in keys {
            let kanjis = merged_dict.get(yomi).unwrap();
            assert!(!yomi.is_empty(), "yomi must not be empty: {kanjis:?}");
            let kanjis = kanjis.join("/");
            wfp.write_fmt(format_args!("{yomi} /{kanjis}/\n"))?;
        }
    }
    Ok(())
}
