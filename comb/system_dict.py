import logging
import os

import marisa_trie
from marisa_trie import BytesTrie

from comb.config import DICTIONARY_DIR


class SystemDict:
    _trie: BytesTrie

    def __init__(self, path, logger=logging.getLogger(__name__)):
        self.logger = logger

        self.logger.info(f"loading cache dictionary: {path}")
        trie = marisa_trie.BytesTrie()
        trie.mmap(path)
        self._trie = trie

    @staticmethod
    def create():
        path = os.path.join(DICTIONARY_DIR, 'system_dict.trie')
        return SystemDict(path)

    def prefixes(self, key):
        return self._trie.prefixes(key)

    def __getitem__(self, item):
        return self._trie[item][0].decode('utf-8').split('/')
