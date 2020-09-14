import math

import marisa_trie

from comb.config import MODEL_DIR
from comb.node import Node

DEFAULT_SCORE = [(math.log10(0.00000000001),)]


class SystemLanguageModel:
    def __init__(self, score: marisa_trie.RecordTrie):
        self.score = score

    @staticmethod
    def create():
        score = marisa_trie.RecordTrie('@f')
        score.mmap(f"{MODEL_DIR}/system_language_model.trie")

        return SystemLanguageModel(score)

    def get_unigram_cost(self, key: str) -> float:
        return self.score.get(key, DEFAULT_SCORE)[0][0]

    def get_bigram_cost(self, node1: Node, node2: Node) -> float:
        key1 = node1.get_key()
        key2 = node2.get_key()
        key = key1 + "\t" + key2
        return self.score.get(key, DEFAULT_SCORE)[0][0]
