use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use anyhow::{bail, Context, Result};
use log::info;
use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Keymap {
    pub extends: Option<String>,
    pub keys: Vec<KeyConfig>,
}

impl Keymap {
    fn to_map(&self) -> Result<HashMap<KeyPattern, Option<String>>> {
        let mut retval = HashMap::new();

        for kc in &self.keys {
            for key in &kc.key {
                let (ctrl, shift, key) = Self::parse_key(key.as_str())?;

                retval.insert(
                    KeyPattern {
                        states: kc.states.clone(),
                        ctrl,
                        shift,
                        key,
                    },
                    kc.command.clone(),
                );
            }
        }

        Ok(retval)
    }

    fn parse_key(key: &str) -> Result<(bool, bool, String)> {
        if key.contains('-') {
            let mut ctrl = false;
            let mut shift = false;
            let keys = key.split('-').collect::<Vec<_>>();
            for m in &keys[0..keys.len() - 1] {
                match *m {
                    "C" => {
                        ctrl = true;
                    }
                    "S" => {
                        shift = true;
                    }
                    _ => {
                        bail!("Unknown modifier in keymap: {}", key);
                    }
                }
            }

            Ok((ctrl, shift, keys[keys.len() - 1].to_string()))
        } else {
            Ok((false, false, key.to_string()))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct KeyConfig {
    pub states: Vec<KeyState>,
    pub key: Vec<String>,
    pub command: Option<String>,
}

// null であとから消すために使う
#[derive(PartialEq, Debug, Hash, Clone)]
pub struct KeyPattern {
    pub states: Vec<KeyState>,
    pub ctrl: bool,
    pub shift: bool,
    pub key: String,
}

impl Eq for KeyPattern {}

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
    pub fn load(keymap_path: &str) -> Result<HashMap<KeyPattern, String>> {
        info!("Load {}", keymap_path);
        let got: Keymap = serde_yaml::from_reader(BufReader::new(
            File::open(keymap_path).with_context(|| keymap_path.to_string())?,
        ))?;

        if let Some(parent) = &got.extends {
            let mut map = Keymap::load(parent.as_str())?;

            for (kp, opts) in &got.to_map()? {
                if let Some(cmd) = opts {
                    // 親の値を上書き
                    map.insert(kp.clone(), cmd.clone());
                } else {
                    // null で親の値を消去できる。
                    map.remove(kp);
                }
            }

            Ok(map)
        } else {
            let got = got
                .to_map()?
                .iter()
                .map(|(a, b)| (a.clone(), b.clone().unwrap()))
                .collect::<HashMap<KeyPattern, String>>();
            Ok(got)
        }
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

    #[test]
    fn test_c_h() -> Result<()> {
        let (ctrl, shift, key) = Keymap::parse_key("C-h")?;
        assert!(ctrl);
        assert!(!shift);
        assert_eq!(key, "h");
        Ok(())
    }

    #[test]
    fn test_c_s_h() -> Result<()> {
        let (ctrl, shift, key) = Keymap::parse_key("C-S-h")?;
        assert!(ctrl);
        assert!(shift);
        assert_eq!(key, "h");
        Ok(())
    }

    #[test]
    fn test_shift() -> Result<()> {
        let (ctrl, shift, key) = Keymap::parse_key("h")?;
        assert!(!ctrl);
        assert!(!shift);
        assert_eq!(key, "h");
        Ok(())
    }
}
