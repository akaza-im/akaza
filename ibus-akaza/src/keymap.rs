use alloc::ffi::CString;
use std::collections::HashMap;

use log::{error, trace};

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
    fn to_ibus_key(s: &str) -> guint {
        let cs = CString::new(s.to_string()).unwrap();
        unsafe { ibus_keyval_from_name(cs.as_ptr()) }
    }

    pub(crate) fn new(keymap_name: Option<String>) -> anyhow::Result<Self> {
        let keymap_name = keymap_name.unwrap_or_else(|| "default".to_string());
        let keymap = Keymap::load(keymap_name.as_str())?;
        let mut mapping: HashMap<KeyPattern, String> = HashMap::new();

        for (key_pattern, command) in keymap {
            let key = &key_pattern.key;
            let mut modifier = 0_u32;
            if key_pattern.ctrl {
                modifier |= IBusModifierType_IBUS_CONTROL_MASK;
            }
            if key_pattern.shift {
                modifier |= IBusModifierType_IBUS_SHIFT_MASK;
            }
            let keyval = Self::to_ibus_key(key.as_str());
            if keyval == IBUS_KEY_VoidSymbol {
                error!("Unknown key symbol: {} {:?}", key, key_pattern);
                continue;
            }
            trace!("Insert: {} {} {} {:?}", modifier, keyval, key, key_pattern);
            for state in &key_pattern.states {
                mapping.insert(KeyPattern::new(*state, keyval, modifier), command.clone());
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
