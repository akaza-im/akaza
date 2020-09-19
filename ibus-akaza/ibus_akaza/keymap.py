from typing import Optional

import gi

gi.require_version('IBus', '1.0')

from gi.repository import IBus

# なにも入力されてない
KEY_STATE_PRECOMPOSITION = 0
# 入力中
KEY_STATE_COMPOSITION = 1
# 変換中
KEY_STATE_CONVERSION = 2


class Keymap:
    def __init__(self):
        self.keys = {
            KEY_STATE_PRECOMPOSITION: {},
            KEY_STATE_COMPOSITION: {},
            KEY_STATE_CONVERSION: {},
        }

    def register_ibus(self, key_state: int, keyval: int, mask: int, method: str):
        self.keys[key_state][keyval] = (method, mask)
        # IBus.ModifierType.CONTROL_MASK

    def register(self, state, key: str, method: str):
        keyval = getattr(IBus, key)
        self.register_ibus(state, keyval, 0, method)

    def get(self, key_state: int, keyval: int, state: int) -> Optional[str]:
        if keyval in self.keys[key_state]:
            got = self.keys[key_state][keyval]
            (got_method, got_mask) = got
            if state ^ got_mask == 0:
                return got_method
        return None
