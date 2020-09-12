import functools
import logging
import math

import marisa_trie

from comb.node import Node
from comb.user_language_model import UserLanguageModel

DEFAULT_SCORE = [(math.log10(0.00000000001),)]


class LanguageModel:
    def __init__(self,
                 system_unigram_score: marisa_trie.RecordTrie,
                 system_bigram_score: marisa_trie.RecordTrie,
                 user_language_model: UserLanguageModel,
                 logger: logging.Logger = logging.getLogger(__name__)):
        self.logger = logger
        self.system_bigram_score = system_bigram_score
        self.system_unigram_score = system_unigram_score
        self.user_language_model = user_language_model

    def calc_node_cost(self, node: Node) -> float:
        if node.is_bos():
            return 0
        elif node.is_eos():
            return 0
        else:
            u = self.user_language_model.get_unigram_cost(node.get_key())
            if u:
                # self.logger.info(f"Use user score: {node.get_key()} -> {u}")
                return u
            return self.system_unigram_score.get(node.get_key(), DEFAULT_SCORE)[0][0]

    @functools.lru_cache
    def calc_bigram_cost(self, prev_node, next_node) -> float:
        # self → node で処理する。
        u = self.user_language_model.get_bigram_cost(prev_node, next_node)
        if u:
            self.logger.info(f"Use user's bigram score: {prev_node.get_key()},{next_node.get_key()} -> {u}")
            return u
        return self.system_bigram_score.get(
            f"{prev_node.get_key()}\t{next_node.get_key()}", DEFAULT_SCORE
        )[0][0]
