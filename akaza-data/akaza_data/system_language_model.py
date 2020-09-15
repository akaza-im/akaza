import math
import pathlib

import marisa_trie

DEFAULT_COST = [(math.log10(0.00000000001),)]


class SystemLanguageModel:
    def __init__(self, score: marisa_trie.RecordTrie, default_cost=None):
        self.default_cost = DEFAULT_COST if default_cost is None else default_cost
        self._score = score

    @staticmethod
    def load(path: str = str(pathlib.Path(__file__).parent.absolute().joinpath('data/system_language_model.trie')),
             default_cost=None):
        score = marisa_trie.RecordTrie('@f')
        score.mmap(path)

        return SystemLanguageModel(
            score=score,
            default_cost=default_cost
        )

    def get_unigram_cost(self, key: str) -> float:
        return self._score.get(key, self.default_cost)[0][0]

    def get_bigram_cost(self, key1: str, key2: str) -> float:
        return self._score.get(key1 + "\t" + key2, self.default_cost)[0][0]

