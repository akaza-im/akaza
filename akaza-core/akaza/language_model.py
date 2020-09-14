import functools
import logging
import math

import marisa_trie

from akaza.node import Node
from akaza.system_language_model import SystemLanguageModel
from akaza.user_language_model import UserLanguageModel



class LanguageModel:
    def __init__(self,
                 system_language_model: SystemLanguageModel,
                 user_language_model: UserLanguageModel,
                 logger: logging.Logger = logging.getLogger(__name__)):
        self.logger = logger
        self.system_language_model = system_language_model
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
            return self.system_language_model.get_unigram_cost(node.get_key())

    @functools.lru_cache
    def calc_bigram_cost(self, prev_node, next_node) -> float:
        # self → node で処理する。
        u = self.user_language_model.get_bigram_cost(prev_node, next_node)
        if u:
            self.logger.info(f"Use user's bigram score: {prev_node.get_key()},{next_node.get_key()} -> {u}")
            return u
        return self.system_language_model.get_bigram_cost(prev_node, next_node)
