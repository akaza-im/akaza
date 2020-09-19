import logging
from typing import Optional, List

import gi

gi.require_version('IBus', '1.0')

from gi.repository import IBus

# なにも入力されてない
KEY_STATE_PRECOMPOSITION = 1
# 入力中
KEY_STATE_COMPOSITION = 2
# 変換中
KEY_STATE_CONVERSION = 4


class Keymap:
    def __init__(self, logger=logging.getLogger(__name__)):
        self.keys = {
            KEY_STATE_PRECOMPOSITION: {},
            KEY_STATE_COMPOSITION: {},
            KEY_STATE_CONVERSION: {},
        }
        self.logger = logger

    def register_ibus(self, key_state: int, keyval: int, mask: int, method: str):
        if keyval not in self.keys[key_state]:
            self.keys[key_state][keyval] = {}
        self.keys[key_state][keyval][mask] = method
        # IBus.ModifierType.CONTROL_MASK

    def do_register(self, state, key: str, method: str):
        mask = 0
        while '-' in key:
            if key.startswith('C-'):
                key = key[2:]
                mask |= int(IBus.ModifierType.CONTROL_MASK)
            elif key.startswith('S-'):
                key = key[2:]
                mask |= int(IBus.ModifierType.SHIFT_MASK)
            else:
                raise RuntimeError(f"Unknown key: {key}")

        keyval = ord(key) if len(key) == 1 else getattr(IBus, key)
        self.register_ibus(state, keyval, mask, method)

    def register(self, states: List[int], keys: List[str], method: str):
        for key in keys:
            for state in states:
                self.do_register(state, key, method)

    def get(self, key_state: int, keyval: int, state: int) -> Optional[str]:
        if keyval in self.keys[key_state]:
            got_method = self.keys[key_state][keyval][state]
            self.logger.debug(f"keyval={keyval}(j:{ord('j')}, J:{ord('J')}) {state} -> {got_method}")
            return got_method
        return None


def build_default_keymap() -> Keymap:
    keymap = Keymap()

    # 入力モードの切り替え
    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_PRECOMPOSITION, KEY_STATE_CONVERSION], ['Henkan'],
                    'set_input_mode_hiragana')
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

    keymap.register([KEY_STATE_CONVERSION], ['space'], 'cursor_down')
    keymap.register([KEY_STATE_COMPOSITION], ['space'], 'update_candidates')

    keymap.register([KEY_STATE_CONVERSION], ['Return', 'KP_Enter'], 'commit_candidate')
    keymap.register([KEY_STATE_COMPOSITION], ['Return', 'KP_Enter'], 'commit_preedit')

    keymap.register([KEY_STATE_COMPOSITION, KEY_STATE_CONVERSION], ['Escape'], 'escape')

    keymap.register([KEY_STATE_CONVERSION], ['BackSpace'], 'erase_character_before_cursor')
    keymap.register([KEY_STATE_COMPOSITION], ['BackSpace'], 'erase_character_before_cursor')

    for n in range(0, 10):
        keymap.register([KEY_STATE_CONVERSION], [str(n), f"KP_{n}"], f"press_number_{n}")

    keymap.register([KEY_STATE_CONVERSION], ['Page_Up', 'KP_Page_Up'], 'page_up')
    keymap.register([KEY_STATE_CONVERSION], ['Page_Down', 'KP_Page_Down'], 'page_down')

    keymap.register([KEY_STATE_CONVERSION], ['Up', 'KP_Up'], 'cursor_up')
    keymap.register([KEY_STATE_CONVERSION], ['Down', 'KP_Down'], 'cursor_down')

    keymap.register([KEY_STATE_CONVERSION], ['Right', 'KP_Right'], 'cursor_right')
    keymap.register([KEY_STATE_CONVERSION], ['S-Right', 'S-KP_Right'], 'extend_clause_right')

    keymap.register([KEY_STATE_CONVERSION], ['Left', 'KP_Left'], 'cursor_left')
    keymap.register([KEY_STATE_CONVERSION], ['S-Left', 'S-KP_Left'], 'extend_clause_left')

    return keymap
