use chrono::Local;
use std::fs;
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

pub fn copy_snapshot(path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all("work/dump/")?;
    fs::copy(
        path,
        Path::new("work/dump/").join(
            Local::now().format("%Y%m%d-%H%M%S").to_string()
                + path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .as_str(),
        ),
    )?;
    Ok(())
}
