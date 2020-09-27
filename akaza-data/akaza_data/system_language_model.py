import math
import pathlib

import marisa_trie

DEFAULT_COST = [(math.log10(1e-20),)]


class SystemLanguageModel:
    def __init__(self,
                 unigram_score: marisa_trie.RecordTrie,
                 bigram_score: marisa_trie.RecordTrie,
                 trigram_score: marisa_trie.RecordTrie,
                 unigram_default_cost,
                 bigram_default_cost,
                 trigram_default_cost
                 ):
        self.unigram_default_cost = unigram_default_cost
        self.bigram_default_cost = bigram_default_cost
        self.trigram_score = trigram_score
        self.unigram_score = unigram_score
        self.bigram_score = bigram_score
        self.trigram_default_cost = trigram_default_cost

    @staticmethod
    def load(
            path_unigram: str = str(
                pathlib.Path(__file__).parent.absolute().joinpath('data/system_language_model.1gram.trie')),
            path_bigram: str = str(
                pathlib.Path(__file__).parent.absolute().joinpath('data/system_language_model.2gram.trie')),
            path_trigram: str = str(
                pathlib.Path(__file__).parent.absolute().joinpath('data/system_language_model.3gram.trie')),
            unigram_default_cost=None, bigram_default_cost=None, trigram_default_cost=None):
        if unigram_default_cost is None:
            unigram_default_cost = DEFAULT_COST
        if bigram_default_cost is None:
            bigram_default_cost = DEFAULT_COST
        if trigram_default_cost is None:
            trigram_default_cost = DEFAULT_COST

        unigram_score = marisa_trie.RecordTrie('<f')
        unigram_score.mmap(path_unigram)

        bigram_score = marisa_trie.RecordTrie('<f')
        bigram_score.mmap(path_bigram)

        trigram_score = marisa_trie.RecordTrie('<f')
        trigram_score.mmap(path_trigram)

        return SystemLanguageModel(
            unigram_score=unigram_score,
            bigram_score=bigram_score,
            trigram_score=trigram_score,
            unigram_default_cost=unigram_default_cost,
            bigram_default_cost=bigram_default_cost,
            trigram_default_cost=trigram_default_cost,
        )

    def get_unigram_cost(self, key: str) -> float:
        return self.unigram_score.get(key, self.unigram_default_cost)[0][0]

    def get_bigram_cost(self, key1: str, key2: str) -> float:
        return self.bigram_score.get(key1 + "\t" + key2, self.bigram_default_cost)[0][0]

    def get_trigram_cost(self, key1: str, key2: str, key3: str) -> float:
        return self.trigram_score.get(key1 + "\t" + key2 + "\t" + key3, self.trigram_default_cost)[0][0]
