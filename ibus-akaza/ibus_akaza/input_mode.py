from typing import List


class InputMode:
    #    input_modes = {
    # u'InputMode.HalfWidthKatakana' : (INPUT_MODE_HALF_WIDTH_KATAKANA, '_ｱ'),
    #        u'InputMode.Latin': (INPUT_MODE_LATIN, '_A'),
    # u'InputMode.WideLatin' : (INPUT_MODE_WIDE_LATIN, 'Ａ'),
    #    }

    def __init__(self, prop_name: str, mode_code: int, symbol: str, label: str):
        self.prop_name = prop_name
        self.mode_code = mode_code
        self.symbol = symbol
        self.label = label

    def __eq__(self, other):
        return self.mode_code == other.mode_code

    def __str__(self):
        return f"<InputMode {self.prop_name}>"


INPUT_MODE_ALNUM = InputMode('InputMode.Alphanumeric', 0, '_A', 'Alphanumeric (C-S-;)')
INPUT_MODE_HIRAGANA = InputMode('InputMode.Hiragana', 1, 'あ', 'ひらがな (C-S-j)')
INPUT_MODE_KATAKANA = InputMode('InputMode.Katakana', 2, 'ア', 'Katakana (C-S-K)')
INPUT_MODE_HALFWIDTH_KATAKANA = InputMode('InputMode.HalfWidthKatakana', 3, '_ｱ', 'Halfwidth Katakana')
INPUT_MODE_FULLWIDTH_ALNUM = InputMode('InputMode.FullWidthAlnum', 4, 'Ａ', 'Fullwidth Alphanumeric (C-S-l)')

_ALL_INPUT_MODE = [
    INPUT_MODE_ALNUM, INPUT_MODE_HIRAGANA, INPUT_MODE_KATAKANA,
    INPUT_MODE_HALFWIDTH_KATAKANA, INPUT_MODE_FULLWIDTH_ALNUM
]

_INPUT_MODE_PROP_NAME2MODE = dict([(mode.prop_name, mode) for mode in _ALL_INPUT_MODE])


def get_all_input_modes() -> List[InputMode]:
    return _ALL_INPUT_MODE


def get_input_mode_from_prop_name(prop_code: str):
    return _INPUT_MODE_PROP_NAME2MODE.get(prop_code, None)
