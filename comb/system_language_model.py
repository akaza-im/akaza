import math

import marisa_trie

from comb.config import MODEL_DIR
from comb.node import Node

DEFAULT_SCORE = [(math.log10(0.00000000001),)]


class SystemLanguageModel:
    def __init__(self, unigram_score: marisa_trie.RecordTrie, bigram_score: marisa_trie.RecordTrie):
        self.unigram_score = unigram_score
        self.bigram_score = bigram_score

    @staticmethod
    def create():
        unigram_score = marisa_trie.RecordTrie('@f')
        unigram_score.mmap(f"{MODEL_DIR}/jawiki.1gram")

        bigram_score = marisa_trie.RecordTrie('@f')
        bigram_score.mmap(f"{MODEL_DIR}/jawiki.2gram")

        return SystemLanguageModel(unigram_score, bigram_score)

    def get_unigram_cost(self, key: str) -> float:
        return self.unigram_score.get(key, DEFAULT_SCORE)[0][0]

    def get_bigram_cost(self, node1: Node, node2: Node) -> float:
        key1 = node1.get_key()
        key2 = node2.get_key()
        key = key1 + "\t" + key2
        return self.bigram_score.get(key, DEFAULT_SCORE)[0][0]
