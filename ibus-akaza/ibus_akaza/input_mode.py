from typing import List


class InputMode:
    #    input_modes = {
    #        u'InputMode.Hiragana': (INPUT_MODE_HIRAGANA, 'あ'),
    # u'InputMode.Katakana' : (INPUT_MODE_KATAKANA, 'ア'),
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


INPUT_MODE_HIRAGANA = InputMode('InputMode.Hiragana', 0, 'あ', 'ひらがな')
INPUT_MODE_LATIN = InputMode('InputMode.Latin', 1, '_A', 'Latin')

_ALL_INPUT_MODE = [INPUT_MODE_HIRAGANA, INPUT_MODE_LATIN]

_INPUT_MODE_PROP_NAME2MODE = dict([(mode.prop_name, mode) for mode in _ALL_INPUT_MODE])


def get_all_input_modes() -> List[InputMode]:
    return _ALL_INPUT_MODE


def get_input_mode_from_prop_name(prop_code: str):
    return _INPUT_MODE_PROP_NAME2MODE.get(prop_code, None)
