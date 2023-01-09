use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn get_file_list(src_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut result: Vec<PathBuf> = Vec::new();

    for src_file in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.metadata().unwrap().is_file())
    {
        result.push(src_file.path().to_path_buf());
    }
    Ok(result)
}
