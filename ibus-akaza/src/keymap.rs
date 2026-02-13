use std::collections::HashMap;
use std::ffi::CString;

use log::{error, trace};

use ibus_sys::core::{IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_SHIFT_MASK};
use ibus_sys::glib::guint;
use ibus_sys::ibus_key::IBUS_KEY_VoidSymbol;
use ibus_sys::keys::ibus_keyval_from_name;
use libakaza::keymap::{KeyPattern, KeyState};

#[derive(Hash, PartialEq, Debug)]
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
        let Ok(cs) = CString::new(s.to_string()) else {
            error!("Key contains NUL byte: {:?}", s);
            return IBUS_KEY_VoidSymbol;
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ibus_key_pattern_equality() {
        // IBusKeyPatternの等価性をテスト
        let pattern1 = IBusKeyPattern::new(KeyState::PreComposition, 97, 0);
        let pattern2 = IBusKeyPattern::new(KeyState::PreComposition, 97, 0);
        let pattern3 = IBusKeyPattern::new(KeyState::PreComposition, 98, 0);

        assert_eq!(pattern1, pattern2, "Identical patterns should be equal");
        assert_ne!(pattern1, pattern3, "Different keyval should not be equal");
    }

    #[test]
    fn test_ibus_key_pattern_different_states() {
        // 異なるKeyStateでパターンが異なることを確認
        let pattern1 = IBusKeyPattern::new(KeyState::PreComposition, 97, 0);
        let pattern2 = IBusKeyPattern::new(KeyState::Composition, 97, 0);

        assert_ne!(
            pattern1, pattern2,
            "Different key states should not be equal"
        );
    }

    #[test]
    fn test_ibus_key_pattern_different_modifiers() {
        // 異なるmodifierでパターンが異なることを確認
        let pattern1 = IBusKeyPattern::new(KeyState::PreComposition, 97, 0);
        let pattern2 = IBusKeyPattern::new(
            KeyState::PreComposition,
            97,
            IBusModifierType_IBUS_CONTROL_MASK,
        );

        assert_ne!(
            pattern1, pattern2,
            "Different modifiers should not be equal"
        );
    }

    #[test]
    fn test_ibus_keymap_new_with_empty_map() {
        // 空のkeymapでIBusKeyMapを作成できることを確認
        let keymap = HashMap::new();
        let result = IBusKeyMap::new(keymap);

        assert!(result.is_ok(), "Should create IBusKeyMap with empty map");
        let ibus_keymap = result.unwrap();
        assert_eq!(
            ibus_keymap.keymap.len(),
            0,
            "Empty keymap should have no entries"
        );
    }

    #[test]
    fn test_ibus_keymap_get_returns_none_for_nonexistent_key() {
        // 存在しないキーの取得でNoneが返されることを確認
        let keymap = HashMap::new();
        let ibus_keymap = IBusKeyMap::new(keymap).unwrap();

        let result = ibus_keymap.get(&KeyState::PreComposition, 97, 0);
        assert!(result.is_none(), "Should return None for nonexistent key");
    }

    #[test]
    fn test_to_ibus_key_with_common_keys() {
        // よく使われるキー名がVoidSymbolではないことを確認
        let space_key = IBusKeyMap::to_ibus_key("space");
        assert_ne!(
            space_key, IBUS_KEY_VoidSymbol,
            "space should be a valid key"
        );

        let return_key = IBusKeyMap::to_ibus_key("Return");
        assert_ne!(
            return_key, IBUS_KEY_VoidSymbol,
            "Return should be a valid key"
        );

        let escape_key = IBusKeyMap::to_ibus_key("Escape");
        assert_ne!(
            escape_key, IBUS_KEY_VoidSymbol,
            "Escape should be a valid key"
        );
    }

    #[test]
    fn test_to_ibus_key_with_invalid_key() {
        // 無効なキー名でVoidSymbolが返されることを確認
        let invalid_key = IBusKeyMap::to_ibus_key("ThisIsNotAValidKeyName12345");
        assert_eq!(
            invalid_key, IBUS_KEY_VoidSymbol,
            "Invalid key should return VoidSymbol"
        );
    }
}
