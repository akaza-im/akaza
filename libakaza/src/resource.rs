use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

pub fn detect_resource_path(base: &str, name: &str) -> anyhow::Result<String> {
    let pathstr: String = if cfg!(test) {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest
            .join("..")
            .join(base)
            .join(name)
            .to_string_lossy()
            .to_string()
    } else {
        let target_path = PathBuf::from(base).join(name);
        let basedirs = crate::xdg_dirs::BaseDirectories::with_prefix("akaza")
            .with_context(|| "Opening xdg directory with 'akaza' prefix")?;
        let pathbuf = basedirs.find_data_file(&target_path);
        let Some(pathbuf) = pathbuf else {
            bail!("Cannot find {:?} in XDG_DATA_HOME or XDG_DATA_DIRS(XDG_DATA_HOME={:?}, XDG_DATA_DIRS={:?}, base={:?}, name={:?})",
                target_path.display(),
                basedirs.get_data_home().to_string_lossy().to_string(),
                basedirs.get_data_dirs().iter().map(|x| x.to_string_lossy().to_string()).collect::<Vec<_>>(),
                base,
                name
            )
        };
        pathbuf.to_string_lossy().to_string()
    };
    Ok(pathstr)
}
