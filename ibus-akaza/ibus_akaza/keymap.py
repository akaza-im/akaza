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
    def __init__(self):
        self.keys = {
            KEY_STATE_PRECOMPOSITION: {},
            KEY_STATE_COMPOSITION: {},
            KEY_STATE_CONVERSION: {},
        }

    def register_ibus(self, key_state: int, keyval: int, mask: int, method: str):
        if keyval not in self.keys[key_state]:
            self.keys[key_state][keyval] = {}
        self.keys[key_state][keyval][mask] = method
        # IBus.ModifierType.CONTROL_MASK

    def register(self, state, key: str, method: str):
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

    def register_multi(self, state, keys: List[str], method: str):
        for key in keys:
            self.register(state, key, method)

    def get(self, key_state: int, keyval: int, state: int) -> Optional[str]:
        if keyval in self.keys[key_state]:
            got_method = self.keys[key_state][keyval][state]
            import logging
            logging.info(f"!!!!!! KJKJKJKJKJK keyval={keyval}(j:{ord('j')}, J:{ord('J')}) {state} (CTRL: {int(IBus.ModifierType.CONTROL_MASK)}) -> {got_method}")
            return got_method
        return None
