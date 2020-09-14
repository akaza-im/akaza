import logging

import marisa_trie
from marisa_trie import BytesTrie


class SystemDict:
    _trie: BytesTrie

    def __init__(self, path, logger=logging.getLogger(__name__)):
        self.logger = logger

        self.logger.info(f"loading cache dictionary: {path}")
        trie = marisa_trie.BytesTrie()
        trie.mmap(path)
        self._trie = trie

    def prefixes(self, key):
        return self._trie.prefixes(key)

    def __getitem__(self, item):
        return self._trie[item][0].decode('utf-8').split('/')
