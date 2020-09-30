import pathlib

import marisa_trie
from marisa_trie import BytesTrie


class EmojiDict:
    _trie: BytesTrie

    def __init__(self, trie: marisa_trie.BytesTrie):
        assert trie is not str
        self._trie = trie

    @staticmethod
    def load(path: str = str(pathlib.Path(__file__).parent.absolute().joinpath('data/emoji.trie'))):
        print(path)
        trie = marisa_trie.BytesTrie()
        trie.mmap(path)
        return EmojiDict(trie)

    def prefixes(self, yomi):
        return self._trie.prefixes(yomi)

    def __getitem__(self, yomi):
        return self._trie[yomi][0].decode('utf-8').split('/')

    def has_item(self, yomi):
        return yomi in self._trie
