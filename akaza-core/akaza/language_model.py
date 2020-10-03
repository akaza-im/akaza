import logging

from akaza.node import Node
from akaza.user_language_model import UserLanguageModel
from akaza_data.system_language_model import SystemLanguageModel


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
            key = node.get_key()
            u = self.user_language_model.get_unigram_cost(key)
            if u is not None:
                # self.logger.info(f"Use user score: {node.get_key()} -> {u}")
                return u
            # print(f"SYSTEM LANGUAGE MODEL UNIGRAM: {key}")
            word_id, score = self.system_language_model.get_unigram_cost(key)
            node.id = word_id
            return score

    def has_unigram_cost_by_yomi(self, yomi: str):
        return self.user_language_model.has_unigram_cost_by_yomi(yomi)

    def calc_bigram_cost(self, prev_node, next_node) -> float:
        # self → node で処理する。
        prev_key = prev_node.get_key()
        next_key = next_node.get_key()
        u = self.user_language_model.get_bigram_cost(prev_key, next_key)
        if u:
            self.logger.info(f"Use user's bigram score: {prev_key},{next_key} -> {u}")
            return u
        return self.system_language_model.get_bigram_cost(prev_node.id, next_node.id)
