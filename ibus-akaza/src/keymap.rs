use alloc::ffi::CString;
use std::collections::HashMap;

use log::{error, trace};

use ibus_sys::core::{IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_SHIFT_MASK};
use ibus_sys::glib::guint;
use ibus_sys::ibus_key::IBUS_KEY_VoidSymbol;
use ibus_sys::keys::ibus_keyval_from_name;
use libakaza::keymap::{KeyPattern, KeyState};

#[derive(Hash, PartialEq)]
struct IBusKeyPattern {
    key_state: KeyState,
    keyval: u32,
    modifier: u32,
}

impl Eq for IBusKeyPattern {}

impl IBusKeyPattern {
    fn new(key_state: KeyState, keyval: u32, modifier: u32) -> Self {
        IBusKeyPattern {
            key_state,
            keyval,
            modifier,
        }
    }
}

pub struct IBusKeyMap {
    keymap: HashMap<IBusKeyPattern, String>,
}

impl IBusKeyMap {
    fn to_ibus_key(s: &str) -> guint {
        let cs = CString::new(s.to_string()).unwrap();
        unsafe { ibus_keyval_from_name(cs.as_ptr()) }
    }

    pub(crate) fn new(keymap: HashMap<KeyPattern, String>) -> anyhow::Result<Self> {
        let mut mapping: HashMap<IBusKeyPattern, String> = HashMap::new();

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
                mapping.insert(
                    IBusKeyPattern::new(*state, keyval, modifier),
                    command.clone(),
                );
            }
        }

        Ok(IBusKeyMap { keymap: mapping })
    }

    pub fn get(&self, key_state: &KeyState, keyval: u32, modifier: u32) -> Option<&String> {
        trace!("MODIFIER: {}", modifier);
        self.keymap
            .get(&IBusKeyPattern::new(*key_state, keyval, modifier))
    }
}
