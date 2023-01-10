use std::collections::HashMap;

use log::trace;

use ibus_sys::core::{IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_SHIFT_MASK};
use ibus_sys::ibus_key::{
    IBUS_KEY_BackSpace, IBUS_KEY_Down, IBUS_KEY_Hangul, IBUS_KEY_Hangul_Hanja, IBUS_KEY_Henkan,
    IBUS_KEY_KP_Down, IBUS_KEY_KP_Enter, IBUS_KEY_KP_Left, IBUS_KEY_KP_Right, IBUS_KEY_KP_Up,
    IBUS_KEY_Left, IBUS_KEY_Muhenkan, IBUS_KEY_Return, IBUS_KEY_Right, IBUS_KEY_Up, IBUS_KEY_h,
    IBUS_KEY_space, IBUS_KEY_0, IBUS_KEY_1, IBUS_KEY_2, IBUS_KEY_3, IBUS_KEY_4, IBUS_KEY_5,
    IBUS_KEY_6, IBUS_KEY_7, IBUS_KEY_8, IBUS_KEY_9, IBUS_KEY_KP_0, IBUS_KEY_KP_1, IBUS_KEY_KP_2,
    IBUS_KEY_KP_3, IBUS_KEY_KP_4, IBUS_KEY_KP_5, IBUS_KEY_KP_6, IBUS_KEY_KP_7, IBUS_KEY_KP_8,
    IBUS_KEY_KP_9,
};

use crate::context::KeyState;

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

struct KeyMapBuilder {
    keymap: HashMap<KeyPattern, String>,
}

impl KeyMapBuilder {
    fn new() -> Self {
        KeyMapBuilder {
            keymap: HashMap::new(),
        }
    }

    fn insert(&mut self, key_states: &[KeyState], keyvals: &[u32], modifier: u32, func_name: &str) {
        for key_state in key_states {
            for keyval in keyvals {
                self.keymap.insert(
                    KeyPattern::new(*key_state, *keyval, modifier),
                    func_name.to_string(),
                );
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



        keymap.register([KEY_STATE_CONVERSION], ['Page_Up', 'KP_Page_Up'], 'page_up')
        keymap.register([KEY_STATE_CONVERSION], ['Page_Down', 'KP_Page_Down'], 'page_down')
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
            0,
            "set_input_mode_hiragana",
        );
        builder.insert(
            &[
                KeyState::Composition,
                KeyState::PreComposition,
                KeyState::Conversion,
            ],
            &[IBUS_KEY_Muhenkan, IBUS_KEY_Hangul_Hanja],
            0,
            "set_input_mode_alnum",
        );

        // basic operations.
        builder.insert(
            &[KeyState::Composition],
            &[IBUS_KEY_space],
            0,
            "update_candidates",
        );
        builder.insert(&[KeyState::Conversion], &[IBUS_KEY_space], 0, "cursor_down");

        builder.insert(
            &[KeyState::Conversion, KeyState::Composition],
            &[IBUS_KEY_BackSpace],
            0,
            "erase_character_before_cursor",
        );
        builder.insert(
            &[KeyState::Conversion, KeyState::Composition],
            &[IBUS_KEY_h],
            IBusModifierType_IBUS_CONTROL_MASK,
            "erase_character_before_cursor",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Return, IBUS_KEY_KP_Enter],
            0,
            "commit_candidate",
        );
        builder.insert(
            &[KeyState::Composition],
            &[IBUS_KEY_Return, IBUS_KEY_KP_Enter],
            0,
            "commit_preedit",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Up, IBUS_KEY_KP_Up],
            0,
            "cursor_up",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Down, IBUS_KEY_KP_Down],
            0,
            "cursor_down",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Right, IBUS_KEY_KP_Right],
            0,
            "cursor_right",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Left, IBUS_KEY_KP_Left],
            0,
            "cursor_left",
        );

        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Right, IBUS_KEY_KP_Right],
            IBusModifierType_IBUS_SHIFT_MASK,
            "extend_clause_right",
        );
        builder.insert(
            &[KeyState::Conversion],
            &[IBUS_KEY_Left, IBUS_KEY_KP_Left],
            IBusModifierType_IBUS_SHIFT_MASK,
            "extend_clause_left",
        );

        let mut num = |keyvals: &[u32], n: i32| {
            // fn insert(&mut self, key_states: &[KeyState], keyvals: &[u32], modifier: u32, func_name: &str) {
            builder.insert(
                &[KeyState::Conversion],
                keyvals,
                0,
                format!("press_number_{}", n).as_str(),
            )
        };
        num(&[IBUS_KEY_1, IBUS_KEY_KP_1], 1);
        num(&[IBUS_KEY_2, IBUS_KEY_KP_2], 2);
        num(&[IBUS_KEY_3, IBUS_KEY_KP_3], 3);
        num(&[IBUS_KEY_4, IBUS_KEY_KP_4], 4);
        num(&[IBUS_KEY_5, IBUS_KEY_KP_5], 5);
        num(&[IBUS_KEY_6, IBUS_KEY_KP_6], 6);
        num(&[IBUS_KEY_7, IBUS_KEY_KP_7], 7);
        num(&[IBUS_KEY_8, IBUS_KEY_KP_8], 8);
        num(&[IBUS_KEY_9, IBUS_KEY_KP_9], 9);
        num(&[IBUS_KEY_0, IBUS_KEY_KP_0], 0);

        KeyMap {
            keymap: builder.keymap,
        }
    }

    pub fn get(&self, key_state: &KeyState, keyval: u32, modifier: u32) -> Option<&String> {
        trace!("MODIFIER: {}", modifier);
        self.keymap
            .get(&KeyPattern::new(*key_state, keyval, modifier))
    }
}
