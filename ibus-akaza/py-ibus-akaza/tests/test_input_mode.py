from ibus_akaza.input_mode import *


def test_input_mode():
    assert INPUT_MODE_HIRAGANA == INPUT_MODE_HIRAGANA
    assert INPUT_MODE_HIRAGANA != INPUT_MODE_ALNUM


def test_input_mode_by_prop_name():
    assert get_input_mode_from_prop_name('InputMode.Hiragana') == INPUT_MODE_HIRAGANA
    assert get_input_mode_from_prop_name('InputMode.Alphanumeric') == INPUT_MODE_ALNUM
