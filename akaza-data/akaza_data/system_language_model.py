import math
import pathlib
from typing import Tuple

from akaza_data.systemlm_loader import SystemLM

DEFAULT_COST = math.log10(1e-20)


class SystemLanguageModel:
    def __init__(self,
                 lm: SystemLM,
                 unigram_default_cost,
                 bigram_default_cost,
                 ):
        self.unigram_default_cost = unigram_default_cost
        self.bigram_default_cost = bigram_default_cost
        self.lm = lm

    @staticmethod
    def load(
            path_unigram: str = str(
                pathlib.Path(__file__).parent.absolute().joinpath('data/lm_v2_1gram.trie')),
            path_bigram: str = str(
                pathlib.Path(__file__).parent.absolute().joinpath('data/lm_v2_2gram.trie')),
            unigram_default_cost=None, bigram_default_cost=None, trigram_default_cost=None):
        if unigram_default_cost is None:
            unigram_default_cost = DEFAULT_COST
        if bigram_default_cost is None:
            bigram_default_cost = DEFAULT_COST

        print(f"[V2] Loading {path_unigram}, {path_bigram}")
        lm = SystemLM()
        lm.load(str(path_unigram), str(path_bigram))

        return SystemLanguageModel(
            lm=lm,
            unigram_default_cost=unigram_default_cost,
            bigram_default_cost=bigram_default_cost,
        )

    def get_unigram_cost(self, key: str) -> Tuple[int, float]:
        id, score = self.lm.find_unigram(key)
        if id >= 0:
            # print(f"UNI HIT: {key}: {id} {score}")
            return id, score
        else:
            # print(f"UNI DEFAULT: {id} {key}")
            return id, self.unigram_default_cost

    def get_bigram_cost(self, id1: int, id2: int) -> float:
        if id1 is None or id2 is None or id1 < 0 or id2 < 0:
            # print(f"BI MISS(NO KEY): {key1} {key2}")
            return self.bigram_default_cost
        score = self.lm.find_bigram(id1, id2)
        # print(f"bigram: id1={id1}, id2={id2} score={score}")
        if score != 0.0:
            # print(f"BI HIT: {key1} {key2} -> {score}")
            return score
        else:
            # print(f"BI MISS: {key1} {key2}")
            return self.bigram_default_cost
