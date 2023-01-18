use std::env;
use std::fs::File;
use std::io::BufReader;

use anyhow::Context;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Keymap {
    pub keys: Vec<KeyConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct KeyConfig {
    pub states: Vec<KeyState>,
    pub key: Vec<String>,
    pub command: String,
}

#[derive(Debug, Hash, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum KeyState {
    // 何も入力されていない状態。
    PreComposition,
    // 変換処理に入る前。ひらがなを入力している段階。
    Composition,
    // 変換中
    Conversion,
}

impl Keymap {
    pub fn load(name: &str) -> anyhow::Result<Keymap> {
        let pathstr: String = if cfg!(test) || cfg!(feature = "it") {
            format!("{}/../keymap/{}.yml", env!("CARGO_MANIFEST_DIR"), name)
        } else if let Ok(env) = env::var("AKAZA_KEYMAP_DIR") {
            format!("{}/{}.yml", env, name)
        } else {
            let pathbuf = xdg::BaseDirectories::with_prefix("akaza")
                .with_context(|| "Opening xdg directory with 'akaza' prefix")?
                .get_config_file(format!("keymap/{}.yml", name));
            pathbuf.to_string_lossy().to_string()
        };
        info!("Load {}", pathstr);
        let got: Keymap = serde_yaml::from_reader(BufReader::new(
            File::open(&pathstr).with_context(|| pathstr)?,
        ))?;
        Ok(got)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_keymap() -> anyhow::Result<()> {
        let keymap: Keymap =
            serde_yaml::from_reader(BufReader::new(File::open("../keymap/default.yml")?))?;
        for kc in keymap.keys {
            println!("{:?}", kc);
        }
        Ok(())
    }
}
