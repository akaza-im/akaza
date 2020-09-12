import logging
import os

import marisa_trie

from comb.config import DICTIONARY_DIR


class SystemDict:
    def __init__(self, logger=logging.getLogger(__name__)):
        self.logger = logger

        path = os.path.join(DICTIONARY_DIR, 'system_dict.trie')
        self.logger.info(f"loading cache dictionary: {path}")
        trie = marisa_trie.BytesTrie()
        trie.mmap(path)
        self.trie = trie

    # src は /better/ みたいな英単語を検索するためにワタシテイルです。
    def get_candidates(self, src, hiragana):
        if src in self.trie:
            kanjis = self.trie[src][0].decode('utf-8').split('/')
            for kanji in kanjis:
                yield kanji

        for prefix in reversed(self.trie.prefixes(hiragana)):
            kanjis = self.trie[prefix][0].decode('utf-8').split('/')
            for kanji in kanjis:
                yield kanji + hiragana[len(prefix):]
