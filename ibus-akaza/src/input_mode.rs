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
    InputMode::new("InputMode.Alphanumeric", 0, "_A", "Alphanumeric (C-S-;)");
pub const INPUT_MODE_HIRAGANA: InputMode =
    InputMode::new("InputMode.Hiragana", 1, "あ", "Hiragana (C-S-j)");
pub const INPUT_MODE_KATAKANA: InputMode =
    InputMode::new("InputMode.Katakana", 2, "ア", "Katakana (C-S-K)");
pub const INPUT_MODE_HALFWIDTH_KATAKANA: InputMode = InputMode::new(
    "InputMode.HalfWidthKatakana",
    3,
    "_ｱ",
    "Half-width Katakana",
);
pub const INPUT_MODE_FULLWIDTH_ALNUM: InputMode = InputMode::new(
    "InputMode.FullWidthAlnum",
    4,
    "Ａ",
    "Full-width Alphanumeric (C-S-l)",
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
