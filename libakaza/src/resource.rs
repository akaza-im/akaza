use anyhow::{bail, Context};
use std::env;

pub fn detect_resource_path(base: &str, env_name: &str, name: &str) -> anyhow::Result<String> {
    let pathstr: String = if cfg!(test) || cfg!(feature = "it") {
        format!("{}/../{}/{}.yml", env!("CARGO_MANIFEST_DIR"), base, name)
    } else if let Ok(env) = env::var(env_name) {
        format!("{}/{}.yml", env, name)
    } else {
        let target_path = format!("{}/{}.yml", base, name);
        let basedirs = xdg::BaseDirectories::with_prefix("akaza")
            .with_context(|| "Opening xdg directory with 'akaza' prefix")?;
        let datadirs = basedirs.list_data_files_once(&target_path);
        let pathbuf = datadirs.first();
        let Some(pathbuf) = pathbuf else {
            bail!("Cannot find {:?} in XDG_DATA_HOME or XDG_DATA_DIRS(XDG_DATA_HOME={:?}, XDG_DATA_DIRS={:?})",
                target_path,
                basedirs.get_data_home().to_string_lossy().to_string(),
                basedirs.get_data_dirs().iter().map(|x| x.to_string_lossy().to_string()).collect::<Vec<_>>(),
            )
        };
        pathbuf.to_string_lossy().to_string()
    };
    Ok(pathstr)
}
