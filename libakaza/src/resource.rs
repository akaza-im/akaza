use std::env;

use anyhow::{bail, Context};

pub fn detect_resource_path(base: &str, env_name: &str, name: &str) -> anyhow::Result<String> {
    let pathstr: String = if cfg!(test) {
        format!("{}/../{}/{}", env!("CARGO_MANIFEST_DIR"), base, name)
    } else if let Ok(env) = env::var(env_name) {
        format!("{}/{}", env, name)
    } else {
        let target_path = format!("{}/{}", base, name);
        let basedirs = xdg::BaseDirectories::with_prefix("akaza")
            .with_context(|| "Opening xdg directory with 'akaza' prefix")?;
        let pathbuf = basedirs.find_data_file(&target_path);
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
