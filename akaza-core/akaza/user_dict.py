import json
from typing import List, Dict

import marisa_trie

from skkdictutils import parse_skkdict, merge_skkdict, ari2nasi


class UserDict:
    _trie: marisa_trie.BytesTrie

    def __init__(self, trie: marisa_trie.BytesTrie):
        self._trie = trie

    def prefixes(self, yomi):
        print(f"UserDict: {yomi}---- prefixes")
        return self._trie.prefixes(yomi)

    def __getitem__(self, item):
        print(f"UserDict: {item}---- __getitem__")
        return self._trie[item][0].decode('utf-8').split('/')

    def has_item(self, item):
        return item in self._trie


def load_user_dict_from_json_config(path: str) -> UserDict:
    # TODO: cache したほうが良さそう。
    with open(path, 'r') as fp:
        conf = json.load(fp)
    dicts = []

    for dictconf in conf:
        path = dictconf['path']
        encoding = dictconf.get('encoding', 'utf-8')
        ari, nasi = parse_skkdict(path, encoding)
        dicts.append(nasi)
        dicts.append(ari2nasi(ari))
    merged = merge_skkdict(dicts)

    t = []
    for k, v in merged.items():
        t.append((k, '/'.join(v).encode('utf-8')))
    trie = marisa_trie.BytesTrie(t)

    return UserDict(trie)
