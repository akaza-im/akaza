use std::collections::HashMap;
use std::ffi::CString;

use anyhow::bail;
use log::{error, info, trace};

use ibus_sys::core::{IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_SHIFT_MASK};
use ibus_sys::glib::guint;
use ibus_sys::ibus_key::IBUS_KEY_VoidSymbol;
use ibus_sys::keys::ibus_keyval_from_name;
use libakaza::keymap::{KeyState, Keymap};

#[derive(Hash, PartialEq)]
struct KeyPattern {
    key_state: KeyState,
    keyval: u32,
    modifier: u32,
}

impl Eq for KeyPattern {}

impl KeyPattern {
    fn new(key_state: KeyState, keyval: u32, modifier: u32) -> Self {
        KeyPattern {
            key_state,
            keyval,
            modifier,
        }
    }
}

pub struct KeyMap {
    keymap: HashMap<KeyPattern, String>,
}

impl KeyMap {
    /// - first: modifier
    /// - second: keyval
    /// ただし、keyval が不明なものの場合は IBUS_KEY_VoidSymbol になる。
    fn split_key(key: &str) -> anyhow::Result<(u32, u32)> {
        fn p(s: &str) -> guint {
            let cs = CString::new(s.to_string()).unwrap();
            unsafe { ibus_keyval_from_name(cs.as_ptr()) }
        }

        let mut modifier = 0_u32;
        if key.contains('-') {
            let keys = key.split('-').collect::<Vec<_>>();
            for m in &keys[0..keys.len() - 1] {
                match *m {
                    "C" => {
                        modifier |= IBusModifierType_IBUS_CONTROL_MASK;
                    }
                    "S" => {
                        modifier |= IBusModifierType_IBUS_SHIFT_MASK;
                    }
                    _ => {
                        bail!("Unknown modifier in keymap: {}", key);
                    }
                }
            }

            Ok((modifier, p(keys[keys.len() - 1])))
        } else {
            Ok((0, p(key)))
        }
    }

    pub(crate) fn new(keymap_name: &str) -> anyhow::Result<Self> {
        // TODO use IBus.Hotkey

        let keymap = Keymap::load(keymap_name)?;
        let mut mapping: HashMap<KeyPattern, String> = HashMap::new();

        for kc in keymap.keys {
            for key in &kc.key {
                let (modifier, keyval) = Self::split_key(key.as_str())?;
                if keyval == IBUS_KEY_VoidSymbol {
                    error!("Unknown key symbol: {} {:?}", key, kc);
                    continue;
                }
                info!("Insert: {} {} {} {:?}", modifier, keyval, key, kc);
                for state in &kc.states {
                    mapping.insert(
                        KeyPattern::new(*state, keyval, modifier),
                        kc.command.clone(),
                    );
                }
            }
        }

        Ok(KeyMap { keymap: mapping })
    }

    pub fn get(&self, key_state: &KeyState, keyval: u32, modifier: u32) -> Option<&String> {
        trace!("MODIFIER: {}", modifier);
        self.keymap
            .get(&KeyPattern::new(*key_state, keyval, modifier))
    }
}

#[cfg(test)]
mod tests {
    use ibus_sys::ibus_key::{IBUS_KEY_Right, IBUS_KEY_h};

    use super::*;

    #[test]
    fn test_c_h() -> anyhow::Result<()> {
        let (modifier, keyval) = KeyMap::split_key("C-h")?;
        assert_eq!(modifier, IBusModifierType_IBUS_CONTROL_MASK);
        assert_eq!(keyval, IBUS_KEY_h);
        info!("Key: C-h, {}, {}", modifier, keyval);
        Ok(())
    }

    #[test]
    fn test_c_s_h() -> anyhow::Result<()> {
        let (modifier, keyval) = KeyMap::split_key("C-S-h")?;
        assert_eq!(
            modifier,
            IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_SHIFT_MASK
        );
        assert_eq!(keyval, IBUS_KEY_h);
        info!("Key: C-S-h, {}, {}", modifier, keyval);
        Ok(())
    }

    #[test]
    fn test_shift() -> anyhow::Result<()> {
        let (modifier, keyval) = KeyMap::split_key("S-Right")?;
        assert_eq!(modifier, IBusModifierType_IBUS_SHIFT_MASK);
        assert_eq!(keyval, IBUS_KEY_Right);
        info!("Key: S-Right, {}, {}", modifier, keyval);
        Ok(())
    }
}
