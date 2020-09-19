import json
from typing import List, Dict

import marisa_trie

from skkdictutils import parse_skkdict, merge_skkdict, ari2nasi


class SkkFileDict:
    """
    ユーザーが設定した辞書。SKK 辞書。
    """
    _trie: marisa_trie.BytesTrie

    def __init__(self, trie: marisa_trie.BytesTrie):
        self._trie = trie

    def prefixes(self, yomi):
        return self._trie.prefixes(yomi)

    def __getitem__(self, yomi):
        return self._trie[yomi][0].decode('utf-8').split('/')

    def has_item(self, yomi):
        return yomi in self._trie


def load_skk_file_dict(path: str, encoding: str = 'utf-8') -> SkkFileDict:
    # TODO: cache したほうが良さそう。
    ari, nasi = parse_skkdict(path, encoding)
    merged = merge_skkdict([
        nasi,
        ari2nasi(ari)
    ])
    print(merged)

    t = []
    for k, v in merged.items():
        t.append((k, '/'.join(v).encode('utf-8')))
    trie = marisa_trie.BytesTrie(t)

    return SkkFileDict(trie)
