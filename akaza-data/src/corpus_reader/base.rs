use std::fs::File;
use std::io::Write;
use std::path::Path;

pub trait CorpusReader {
    fn process_file<F>(&self, ifname: &Path, ofname: &Path, annotate: &mut F) -> anyhow::Result<()>
    where
        F: FnMut(&str) -> anyhow::Result<String>;
}

/// _SUCCESS ファイルを書く
pub fn write_success_file(dst_dir: &Path) -> anyhow::Result<()> {
    let mut success = File::create(dst_dir.join("_SUCCESS"))?;
    success.write_all("DONE".as_bytes())?;
    Ok(())
}
