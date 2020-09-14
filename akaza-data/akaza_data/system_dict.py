import pathlib

import marisa_trie
from marisa_trie import BytesTrie


class SystemDict:
    _trie: BytesTrie

    def __init__(self, trie: marisa_trie.BytesTrie):
        self._trie = trie

    @staticmethod
    def load(path: str = str(pathlib.Path(__file__).parent.absolute().joinpath('data/system_dict.trie'))):
        print(path)
        trie = marisa_trie.BytesTrie()
        trie.mmap(path)
        return SystemDict(trie)

    def prefixes(self, key):
        return self._trie.prefixes(key)

    def __getitem__(self, item):
        return self._trie[item][0].decode('utf-8').split('/')
