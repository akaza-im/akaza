import logging
import math

from akaza.node import Node
from akaza.user_language_model import UserLanguageModel
from akaza_data.systemlm_loader import SystemLM

UNIGRAM_DEFAULT_COST = math.log10(1e-20)
BIGRAM_DEFAULT_COST = math.log10(1e-20)


class LanguageModel:
    def __init__(self,
                 system_language_model: SystemLM,
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
            word_id, score = self.system_language_model.find_unigram(key)
            if word_id < 0:
                score = UNIGRAM_DEFAULT_COST
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

        id1 = prev_node.id
        id2 = next_node.id
        if id1 is None or id2 is None or id1 < 0 or id2 < 0:
            # print(f"BI MISS(NO KEY): {key1} {key2}")
            return BIGRAM_DEFAULT_COST
        score = self.system_language_model.find_bigram(id1, id2)

        # print(f"bigram: id1={id1}, id2={id2} score={score}")
        if score != 0.0:
            # print(f"BI HIT: {key1} {key2} -> {score}")
            return score
        else:
            # print(f"BI MISS: {key1} {key2}")
            return BIGRAM_DEFAULT_COST
