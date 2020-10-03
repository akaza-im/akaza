import math
import pathlib

from akaza_data.systemlm_loader import SystemLM

DEFAULT_COST = [(math.log10(1e-20),)]


class SystemLanguageModelV2:
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

        lm = SystemLM()
        lm.load(str(path_unigram), str(path_bigram))

        return SystemLanguageModelV2(
            lm=lm,
            unigram_default_cost=unigram_default_cost,
            bigram_default_cost=bigram_default_cost,
        )

    def get_unigram_cost(self, key: str) -> float:
        id, score = self.lm.find_unigram(key)
        if id >= 0:
            return score
        else:
            return self.unigram_default_cost[0][0]

    def get_bigram_cost(self, key1: str, key2: str) -> float:
        # TODO: optimize
        id1, _ = self.lm.find_unigram(key1)
        id2, _ = self.lm.find_unigram(key2)
        if id1 < 0 or id2 < 0:
            return self.bigram_default_cost[0][0]
        score = self.lm.find_bigram(id1, id2)
        print(f"bigram: id1={id1}, id2={id2} score={score}")
        if score > 0.0:
            return score
        else:
            return self.bigram_default_cost[0][0]
