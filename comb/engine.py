from typing import List, Any

import os

import jaconv

from comb import combromkan

from comb.system_dict import SystemDict
from comb.user_dict import UserDict


class Comb:
    dictionaries: List[Any]

    def __init__(self, logger, user_dict: UserDict, system_dict: SystemDict):
        self.logger = logger
        self.dictionaries = []
        self.user_dict = user_dict
        self.system_dict = system_dict

    def convert(self, src):
        hiragana: str = combromkan.to_hiragana(src)
        katakana: str = jaconv.hira2kata(hiragana)

        candidates = [[hiragana, hiragana]]

        for e in self.user_dict.get_candidates(src, hiragana):
            if e not in candidates:
                candidates.append(e)

        if [katakana, katakana] not in candidates:
            candidates.append([katakana, katakana])

        for e in [[x, x] for x in self.system_dict.get_candidates(src, hiragana)]:
            if e not in candidates:
                candidates.append(e)

        if src[0].isupper():
            # 先頭が大文字の場合、それを先頭にもってくる。
            candidates.insert(0, [src, src])
        else:
            # そうじゃなければ、末尾にいれる。
            candidates.append([src, src])

        return candidates


if __name__ == '__main__':
    from gi.repository import GLib
    import pathlib
    import logging

    logging.basicConfig(level=logging.DEBUG)

    configdir = os.path.join(GLib.get_user_config_dir(), 'ibus-comb')
    pathlib.Path(configdir).mkdir(parents=True, exist_ok=True)
    d = SystemDict()
    u = UserDict(os.path.join(configdir, 'user-dict.txt'))
    comb = Comb(logging.getLogger(__name__), u, d)
    # print(comb.convert('henkandekiru'))
    print(comb.convert('watasi'))
    # print(comb.convert('hituyoudayo'))
    # print(list(d.get_candidates('henkandekiru', 'へんかんできる')))
    # print(list(d.get_candidates('warudakumi', 'わるだくみ')))
    # print(list(d.get_candidates('subarasii', 'すばらしい')))
    # print(list(d.get_candidates('watasi', 'わたし')))
    # print(list(d.get_candidates('hiragana', 'ひらがな')))
    # print(list(d.get_candidates('buffer', 'ぶっふぇr')))
