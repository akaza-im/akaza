import logging
import math
import os
import time
from typing import List, Dict, Optional, Set

from atomicwrites import atomic_write

from akaza.node import Node

# additive factor
ALPHA = 0.00001


# ユーザーの言語モデル。
#
# unigram score
# bigram score
class UserLanguageModel:
    unigram_kanas: Set[str]
    unigram: Dict[str, int]

    def __init__(self, path: str, logger=logging.getLogger(__name__)):
        self.path = path
        self.logger = logger

        self.need_save = False

        self.unigram_kanas = set()

        if os.path.exists(self.unigram_path()):
            self.unigram_C, self.unigram_V, self.unigram = self.read(self.unigram_path(), is_unigram=True)
        else:
            self.unigram_C, self.unigram_V, self.unigram = 0, 0, {}

        if os.path.exists(self.bigram_path()):
            self.bigram_C, self.bigram_V, self.bigram = self.read(self.bigram_path())
        else:
            self.bigram_C, self.bigram_V, self.bigram = 0, 0, {}

    def unigram_path(self):
        return os.path.join(self.path, '1gram.txt')

    def bigram_path(self):
        return os.path.join(self.path, '2gram.txt')

    def read(self, path, is_unigram=False):
        # 単語数
        V = 0
        # 総単語出現数
        C = 0
        word_data = {}
        with open(path) as fp:
            for line in fp:
                m = line.rstrip().split(" ")
                if len(m) == 2:
                    key, count = m
                    count = int(count)
                    word_data[key] = count
                    if is_unigram:
                        kanji, kana = key.split('/')
                        self.unigram_kanas.add(kana)
                    V += 1
                    C += count
        return V, C, word_data

    def add_entry(self, nodes: List[Node]):
        # unigram
        for node in nodes:
            key = node.get_key()

            self.logger.info(f"add user_language_model entry: key={key}")

            if key not in self.unigram:
                self.unigram_C += 1
            self.unigram_V += 1
            kanji, kana = key.split('/')
            self.unigram_kanas.add(kana)
            self.unigram[key] = self.unigram.get(key, 0) + 1

        # bigram
        for i in range(1, len(nodes)):
            node1 = nodes[i - 1]
            node2 = nodes[i]
            key = node1.get_key() + "\t" + node2.get_key()
            if key not in self.bigram:
                self.bigram_C += 1
            self.bigram_V += 1
            self.bigram[key] = self.bigram.get(key, 0) + 1

        self.need_save = True

    def save(self):
        if not self.need_save:
            self.logger.debug("Skip saving user_language_mdel.")
            return

        self.need_save = False
        self.logger.info("Writing user_language_model")
        with atomic_write(self.unigram_path(), overwrite=True) as f:
            for words in sorted(self.unigram.keys()):
                count = self.unigram[words]
                f.write(f"{words} {count}\n")

        with atomic_write(self.bigram_path(), overwrite=True) as f:
            for words in sorted(self.bigram.keys()):
                count = self.bigram[words]
                f.write(f"{words} {count}\n")

        self.logger.info(f"SAVED {self.path}")

    def get_unigram_cost(self, key: str) -> Optional[float]:
        if key in self.unigram:
            count = self.unigram[key]
            return math.log10((count + ALPHA) / (self.unigram_C + ALPHA * self.unigram_V))
        return None

    def has_unigram_cost_by_yomi(self, yomi: str) -> bool:
        return yomi in self.unigram_kanas

    def get_bigram_cost(self, key1: str, key2: str) -> Optional[float]:
        key = f"{key1}\t{key2}"
        if key in self.bigram:
            count = self.bigram[key]
            return math.log10((count + ALPHA) / (self.bigram_C + ALPHA * self.bigram_V))
        return None

    def save_periodically(self):
        while True:
            self.save()
            time.sleep(60)
