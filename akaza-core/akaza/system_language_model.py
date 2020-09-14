import math

import marisa_trie

from akaza.node import Node

DEFAULT_SCORE = [(math.log10(0.00000000001),)]


class SystemLanguageModel:
    def __init__(self, score: marisa_trie.RecordTrie, default_score=None):
        self.default_score = DEFAULT_SCORE if default_score is None else default_score
        self.score = score

    @staticmethod
    def create(path: str, default_score=None):
        score = marisa_trie.RecordTrie('@f')
        score.mmap(path)

        return SystemLanguageModel(
            score=score,
            default_score=DEFAULT_SCORE if default_score is None else default_score
        )

    def get_unigram_cost(self, key: str) -> float:
        return self.score.get(key, self.default_score)[0][0]

    def get_bigram_cost(self, key1: str, key2: str) -> float:
        return self.score.get(key1 + "\t" + key2, self.default_score)[0][0]
