import logging
import math
import os
from typing import List, Dict, Optional

from atomicwrites import atomic_write

from comb.node import Node


# ユーザー辞書。
#
# カタカナなどの単語の追加辞書。
# unigram score
# bigram score
class UserDict:
    unigram: Dict[str, int]

    def __init__(self, path, logger=logging.getLogger(__name__)):
        self.path = path
        self.logger = logger

        self.unigram = {}
        if os.path.exists(self.unigram_path()):
            self.read()
        else:
            self.total = 0

    def unigram_path(self):
        return os.path.join(self.path, 'unigram.txt')

    def read(self):
        total = 0
        with open(self.unigram_path()) as fp:
            for line in fp:
                m = line.rstrip().split(" ")
                if len(m) == 2:
                    kanji_kana, count = m
                    self.unigram[kanji_kana] = count
                    total += count
            self.total = total

    def add_entry(self, nodes: List[Node]):
        for node in nodes:
            kanji = node.word
            kana = node.yomi

            self.logger.info(f"add user_dict entry: kana='{kana}' kanji='{kanji}'")

            key = f"{kanji}/{kana}"
            self.unigram[key] = self.unigram.get(key, 0) + 1
            self.total += 1

    def save(self):
        with atomic_write(self.unigram_path(), overwrite=True) as f:
            for kanji_kana in sorted(self.unigram.keys()):
                count = self.unigram[kanji_kana]
                f.write(f"{kanji_kana}\t{count}\n")
        self.logger.info(f"SAVED {self.path}")

    def get_unigram_cost(self, key: str) -> Optional[float]:
        if key in self.unigram:
            count = self.unigram[key]
            return math.log10(count / self.total)
        return None
