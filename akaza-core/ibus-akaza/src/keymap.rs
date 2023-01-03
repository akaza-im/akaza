use crate::KeyState;
use ibus_sys::ibus_key::{IBUS_KEY_BackSpace, IBUS_KEY_KP_Enter, IBUS_KEY_Return};
use std::collections::HashMap;

#[derive(Hash, PartialEq)]
struct KeyPattern {
    key_state: KeyState,
    keyval: u32,
}

impl Eq for KeyPattern {}

impl KeyPattern {
    fn new(key_state: KeyState, keyval: u32) -> Self {
        KeyPattern { key_state, keyval }
    }
}

struct KeyMapBuilder {
    keymap: HashMap<KeyPattern, String>,
}
impl KeyMapBuilder {
    fn new() -> Self {
        KeyMapBuilder {
            keymap: HashMap::new(),
        }
    }

    fn insert(&mut self, key_states: &[KeyState], keyvals: &[u32], func_name: &str) {
        for key_state in key_states {
            for keyval in keyvals {
                self.keymap
                    .insert(KeyPattern::new(*key_state, *keyval), func_name.to_string());
            }
        }
    }
}

pub struct KeyMap {
    keymap: HashMap<KeyPattern, String>,
}
impl KeyMap {
    pub(crate) fn new() -> Self {
        let mut builder = KeyMapBuilder::new();

        builder.insert(
            &[KeyState::Conversion, KeyState::Composition],
            &[IBUS_KEY_BackSpace],
            "erase_character_before_cursor",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Return, IBUS_KEY_KP_Enter],
            "commit_candidate",
        );
        builder.insert(
            &[KeyState::Composition],
            &[IBUS_KEY_Return, IBUS_KEY_KP_Enter],
            "commit_preedit",
        );

        KeyMap {
            keymap: builder.keymap,
        }
    }

    pub fn get(&self, key_state: &KeyState, keyval: u32) -> Option<&String> {
        self.keymap.get(&KeyPattern::new(*key_state, keyval))
    }
}
