use std::collections::HashMap;

use crate::context::KeyState;
use ibus_sys::ibus_key::{
    IBUS_KEY_BackSpace, IBUS_KEY_Hangul, IBUS_KEY_Hangul_Hanja, IBUS_KEY_Henkan, IBUS_KEY_KP_Enter,
    IBUS_KEY_KP_Left, IBUS_KEY_KP_Right, IBUS_KEY_Left, IBUS_KEY_Muhenkan, IBUS_KEY_Return,
    IBUS_KEY_Right, IBUS_KEY_space,
};

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

        /*
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-J'],
                        'set_input_mode_hiragana')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['Muhenkan'],
                        'set_input_mode_alnum')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-:'],
                        'set_input_mode_alnum')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-L'],
                        'set_input_mode_fullwidth_alnum')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['C-S-K'],
                        'set_input_mode_katakana')

        # 後から文字タイプを指定する
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F6'], 'convert_to_full_hiragana')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F7'], 'convert_to_full_katakana')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F8'], 'convert_to_half_katakana')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F9'], 'convert_to_full_romaji')
        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['F10'], 'convert_to_half_romaji')

        keymap.register([KEY_STATE_CONVERSION], ['Return', 'KP_Enter'], 'commit_candidate')

        keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['Escape'], 'escape')


        for n in range(0, 10):
            keymap.register([KEY_STATE_CONVERSION], [str(n), f"KP_{n}"], f"press_number_{n}")

        keymap.register([KEY_STATE_CONVERSION], ['Page_Up', 'KP_Page_Up'], 'page_up')
        keymap.register([KEY_STATE_CONVERSION], ['Page_Down', 'KP_Page_Down'], 'page_down')

        keymap.register([KEY_STATE_CONVERSION], ['Up', 'KP_Up'], 'cursor_up')
        keymap.register([KEY_STATE_CONVERSION], ['Down', 'KP_Down'], 'cursor_down')

        keymap.register([KEY_STATE_CONVERSION], ['S-Right', 'S-KP_Right'], 'extend_clause_right')

        keymap.register([KEY_STATE_CONVERSION], ['S-Left', 'S-KP_Left'], 'extend_clause_left')
         */

        // TODO make this configurable.
        // TODO use IBus.Hotkey

        // 入力モードの切り替え
        builder.insert(
            &[
                KeyState::Composition,
                KeyState::PreComposition,
                KeyState::Conversion,
            ],
            &[IBUS_KEY_Henkan, IBUS_KEY_Hangul],
            "set_input_mode_hiragana",
        );
        builder.insert(
            &[
                KeyState::Composition,
                KeyState::PreComposition,
                KeyState::Conversion,
            ],
            &[IBUS_KEY_Muhenkan, IBUS_KEY_Hangul_Hanja],
            "set_input_mode_alnum",
        );

        // basic operations.
        builder.insert(
            &[KeyState::Composition],
            &[IBUS_KEY_space],
            "update_candidates",
        );
        builder.insert(&[KeyState::Conversion], &[IBUS_KEY_space], "cursor_down");

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
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Right, IBUS_KEY_KP_Right],
            "cursor_right",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Left, IBUS_KEY_KP_Left],
            "cursor_left",
        );

        KeyMap {
            keymap: builder.keymap,
        }
    }

    pub fn get(&self, key_state: &KeyState, keyval: u32) -> Option<&String> {
        self.keymap.get(&KeyPattern::new(*key_state, keyval))
    }
}
