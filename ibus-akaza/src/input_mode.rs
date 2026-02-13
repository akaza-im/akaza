use anyhow::bail;

#[derive(Copy, Clone, Debug)]
pub struct InputMode {
    pub prop_name: &'static str,
    pub mode_code: i32,
    pub symbol: &'static str,
    pub label: &'static str,
}

impl PartialEq for InputMode {
    fn eq(&self, other: &Self) -> bool {
        self.mode_code == other.mode_code
    }
}

impl InputMode {
    const fn new(
        prop_name: &'static str,
        mode_code: i32,
        symbol: &'static str,
        label: &'static str,
    ) -> InputMode {
        InputMode {
            prop_name,
            mode_code,
            symbol,
            label,
        }
    }
}

pub const INPUT_MODE_ALNUM: InputMode =
    InputMode::new("InputMode.Alphanumeric", 0, "_A", "アルファベット (C-S-;)");
pub const INPUT_MODE_HIRAGANA: InputMode =
    InputMode::new("InputMode.Hiragana", 1, "あ", "ひらがな (C-S-j)");
pub const INPUT_MODE_KATAKANA: InputMode =
    InputMode::new("InputMode.Katakana", 2, "ア", "カタカナ (C-S-K)");
pub const INPUT_MODE_HALFWIDTH_KATAKANA: InputMode =
    InputMode::new("InputMode.HalfWidthKatakana", 3, "_ｱ", "半角カタカナ");
pub const INPUT_MODE_FULLWIDTH_ALNUM: InputMode = InputMode::new(
    "InputMode.FullWidthAlnum",
    4,
    "Ａ",
    "全角アルファベット (C-S-l)",
);

const _ALL_INPUT_MODE: [InputMode; 5] = [
    INPUT_MODE_ALNUM,
    INPUT_MODE_HIRAGANA,
    INPUT_MODE_KATAKANA,
    INPUT_MODE_HALFWIDTH_KATAKANA,
    INPUT_MODE_FULLWIDTH_ALNUM,
];

pub fn get_all_input_modes() -> &'static [InputMode; 5] {
    &_ALL_INPUT_MODE
}

pub fn get_input_mode_from_prop_name(prop_code: &str) -> anyhow::Result<InputMode> {
    for mode in _ALL_INPUT_MODE {
        if mode.prop_name == prop_code {
            return Ok(mode);
        }
    }
    bail!("Unknown prop_code: {}", prop_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_input_modes_have_unique_mode_codes() {
        // 全てのInputModeが一意なmode_codeを持つことを確認
        let modes = get_all_input_modes();
        let mut codes = std::collections::HashSet::new();

        for mode in modes {
            assert!(
                codes.insert(mode.mode_code),
                "Duplicate mode_code found: {}",
                mode.mode_code
            );
        }

        assert_eq!(codes.len(), 5, "Should have 5 unique mode codes");
    }

    #[test]
    fn test_get_all_input_modes_returns_five_modes() {
        // get_all_input_modes が5つのモードを返すことを確認
        let modes = get_all_input_modes();
        assert_eq!(modes.len(), 5, "Should return 5 input modes");
    }

    #[test]
    fn test_get_input_mode_from_prop_name_valid() {
        // 有効なprop_nameで正しいモードが取得できることを確認
        let mode = get_input_mode_from_prop_name("InputMode.Hiragana").unwrap();
        assert_eq!(mode, INPUT_MODE_HIRAGANA);
        assert_eq!(mode.mode_code, 1);

        let mode = get_input_mode_from_prop_name("InputMode.Alphanumeric").unwrap();
        assert_eq!(mode, INPUT_MODE_ALNUM);
        assert_eq!(mode.mode_code, 0);

        let mode = get_input_mode_from_prop_name("InputMode.Katakana").unwrap();
        assert_eq!(mode, INPUT_MODE_KATAKANA);
        assert_eq!(mode.mode_code, 2);

        let mode = get_input_mode_from_prop_name("InputMode.HalfWidthKatakana").unwrap();
        assert_eq!(mode, INPUT_MODE_HALFWIDTH_KATAKANA);
        assert_eq!(mode.mode_code, 3);

        let mode = get_input_mode_from_prop_name("InputMode.FullWidthAlnum").unwrap();
        assert_eq!(mode, INPUT_MODE_FULLWIDTH_ALNUM);
        assert_eq!(mode.mode_code, 4);
    }

    #[test]
    fn test_get_input_mode_from_prop_name_invalid() {
        // 無効なprop_nameでエラーが返されることを確認
        let result = get_input_mode_from_prop_name("InvalidMode");
        assert!(result.is_err(), "Should return error for invalid prop name");

        let result = get_input_mode_from_prop_name("");
        assert!(result.is_err(), "Should return error for empty prop name");
    }

    #[test]
    fn test_input_mode_equality() {
        // PartialEqの実装が正しく動作することを確認
        assert_eq!(INPUT_MODE_HIRAGANA, INPUT_MODE_HIRAGANA);
        assert_ne!(INPUT_MODE_HIRAGANA, INPUT_MODE_KATAKANA);
        assert_ne!(INPUT_MODE_ALNUM, INPUT_MODE_FULLWIDTH_ALNUM);
    }

    #[test]
    fn test_input_mode_constants_have_correct_fields() {
        // 各InputMode定数が正しいフィールドを持つことを確認
        assert_eq!(INPUT_MODE_HIRAGANA.prop_name, "InputMode.Hiragana");
        assert_eq!(INPUT_MODE_HIRAGANA.mode_code, 1);
        assert_eq!(INPUT_MODE_HIRAGANA.symbol, "あ");
        assert!(!INPUT_MODE_HIRAGANA.label.is_empty());

        assert_eq!(INPUT_MODE_ALNUM.prop_name, "InputMode.Alphanumeric");
        assert_eq!(INPUT_MODE_ALNUM.mode_code, 0);
        assert_eq!(INPUT_MODE_ALNUM.symbol, "_A");
        assert!(!INPUT_MODE_ALNUM.label.is_empty());
    }
}
