import math
import pathlib

import marisa_trie

DEFAULT_SCORE = [(math.log10(0.00000000001),)]


class SystemLanguageModel:
    def __init__(self, score: marisa_trie.RecordTrie, default_score=None):
        self.default_score = DEFAULT_SCORE if default_score is None else default_score
        self.score = score

    @staticmethod
    def load(path: str = str(pathlib.Path(__file__).parent.absolute().joinpath('data/system_dict.trie')),
             default_score=None):
        score = marisa_trie.RecordTrie('@f')
        score.mmap(path)

        return SystemLanguageModel(
            score=score,
            default_score=default_score
        )

    def get_unigram_cost(self, key: str) -> float:
        return self.score.get(key, self.default_score)[0][0]

    def get_bigram_cost(self, key1: str, key2: str) -> float:
        return self.score.get(key1 + "\t" + key2, self.default_score)[0][0]
