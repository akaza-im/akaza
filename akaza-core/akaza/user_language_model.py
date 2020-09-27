import logging
import math
import os
import time
from typing import List, Dict, Optional, Set, Any

from atomicwrites import atomic_write

from akaza.node import Node


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

        self.unigram = {}
        self.unigram_kanas = set()
        if os.path.exists(self.unigram_path()):
            self.read_unigram()
        else:
            self.total = 0

        self.bigram = {}
        self.bigram_total = {}
        if os.path.exists(self.bigram_path()):
            self.read_bigram()

        self.trigram = {}
        self.trigram_total = {}
        if os.path.exists(self.trigram_path()):
            self.read_trigram()

    def unigram_path(self):
        return os.path.join(self.path, '1gram.txt')

    def bigram_path(self):
        return os.path.join(self.path, '2gram.txt')

    def trigram_path(self):
        return os.path.join(self.path, '3gram.txt')

    def read_unigram(self):
        total = 0
        with open(self.unigram_path()) as fp:
            for line in fp:
                m = line.rstrip().split(" ")
                if len(m) == 2:
                    kanji_kana, count = m
                    kanji, kana = kanji_kana.split('/')
                    self.unigram_kanas.add(kana)
                    count = int(count)
                    self.unigram[kanji_kana] = count
                    total += count
            self.total = total

    def read_bigram(self):
        with open(self.bigram_path()) as fp:
            for line in fp:
                m = line.rstrip().split(" ")
                if len(m) == 2:
                    words, count = m
                    count = int(count)
                    self.bigram[words] = count
                    minus1 = "\t".join(words.split("\t")[:-1])
                    self.bigram_total[minus1] = self.bigram_total.get(minus1, 0) + 1

    def read_trigram(self):
        with open(self.trigram_path()) as fp:
            for line in fp:
                m = line.rstrip().split(" ")
                if len(m) == 2:
                    # words: tab separeted
                    words, count = m
                    count = int(count)
                    self.trigram[words] = count
                    minus1 = "\t".join(words.split("\t")[:-1])
                    self.trigram_total[minus1] = self.trigram_total.get(minus1, 0) + 1

    def add_entry(self, nodes: List[Node]):
        # unigram
        for node in nodes:
            kanji = node.word
            kana = node.yomi

            key = node.get_key()

            self.logger.info(f"add user_language_model entry: kana='{kana}' kanji='{kanji}' key={key}")
            print(f"add user_language_model entry: key={key}")

            self.unigram_kanas.add(kana)

            self.unigram[key] = self.unigram.get(key, 0) + 1
            self.total += 1

        # bigram
        for i in range(1, len(nodes)):
            node1 = nodes[i - 1]
            node2 = nodes[i]
            key = node1.get_key() + "\t" + node2.get_key()
            self.bigram[key] = self.bigram.get(key, 0) + 1
            self.bigram_total[node1.get_key()] = self.bigram_total.get(node1.get_key(), 0) + 1

        # trigram
        for i in range(2, len(nodes)):
            node1 = nodes[i - 2]
            node2 = nodes[i - 1]
            node3 = nodes[i]
            key = node1.get_key() + "\t" + node2.get_key() + "\t" + node3.get_key()
            self.trigram[key] = self.trigram.get(key, 0) + 1
            minus1 = node1.get_key() + "\t" + node2.get_key()
            self.trigram_total[minus1] = self.trigram_total.get(minus1, 0) + 1

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

        with atomic_write(self.trigram_path(), overwrite=True) as f:
            for words in sorted(self.trigram.keys()):
                count = self.trigram[words]
                f.write(f"{words} {count}\n")

        self.logger.info(f"SAVED {self.path}")

    def get_unigram_cost(self, key: str) -> Optional[float]:
        if key in self.unigram:
            count = self.unigram[key]
            return math.log10(count / self.total)
        return None

    def has_unigram_cost_by_yomi(self, yomi: str) -> bool:
        return yomi in self.unigram_kanas

    def get_bigram_cost(self, key1: str, key2: str) -> Optional[float]:
        key = key1 + "\t" + key2
        if key in self.bigram:
            count = self.bigram[key]
            return math.log10(count / self.bigram_total[key1])
        return None

    def get_trigram_cost(self, key1: str, key2: str, key3: str) -> Optional[float]:
        key = key1 + "\t" + key2 + "\t" + key3
        if key in self.trigram:
            count = self.trigram[key]
            return math.log10(count / self.trigram_total[key1 + "\t" + key2])
        return None

    def save_periodically(self):
        while True:
            self.save()
            time.sleep(60)
