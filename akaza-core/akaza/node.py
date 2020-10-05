import math
from typing import Optional

from akaza import tinylisp

UNIGRAM_DEFAULT_COST = math.log10(1e-20)
BIGRAM_DEFAULT_COST = math.log10(1e-20)


class AbstractNode:
    id: Optional[int]

    def __init__(self):
        self._bigram_cache = {}
        self.id = None

    def is_eos(self):
        raise NotImplemented()

    def is_bos(self):
        raise NotImplemented()

    @staticmethod
    def _calc_bigram_cost(prev_node, next_node, user_language_model, system_language_model) -> float:
        # self → node で処理する。
        prev_key = prev_node.get_key()
        next_key = next_node.get_key()
        u = user_language_model.get_bigram_cost(prev_key, next_key)
        if u:
            return u

        id1 = prev_node.id
        id2 = next_node.id
        if id1 is None or id2 is None or id1 < 0 or id2 < 0:
            # print(f"BI MISS(NO KEY): {key1} {key2}")
            return BIGRAM_DEFAULT_COST
        score = system_language_model.find_bigram(id1, id2)

        # print(f"bigram: id1={id1}, id2={id2} score={score}")
        if score != 0.0:
            # print(f"BI HIT: {key1} {key2} -> {score}")
            return score
        else:
            # print(f"BI MISS: {key1} {key2}")
            return BIGRAM_DEFAULT_COST

    def get_bigram_cost(self, next_node, user_language_model, system_language_model):
        next_node_key = next_node.get_key()
        if next_node_key in self._bigram_cache:
            return self._bigram_cache[next_node_key]
        else:
            cost = self._calc_bigram_cost(self, next_node, user_language_model, system_language_model)
            self._bigram_cache[next_node_key] = cost
            return cost

    def calc_node_cost(self, user_language_model, system_language_model):
        raise NotImplemented()


class BosNode(AbstractNode):
    def __init__(self):
        super().__init__()
        self.start_pos = -9999
        self.word = '__BOS__'
        self.yomi = '__BOS__'
        self.prev = None
        self.cost = 0

    def is_bos(self):
        return True

    def is_eos(self):
        return False

    def get_key(self):
        return '__BOS__/__BOS__'

    def surface(self, evaluator: tinylisp.Evaluator):
        return '__BOS__'

    def __repr__(self):
        return f"<BosNode: start_pos={self.start_pos}, prev={self.prev.word if self.prev else '-'}>"

    def calc_node_cost(self, user_language_model, system_language_model):
        return 0


class EosNode(AbstractNode):
    def __init__(self, start_pos):
        super().__init__()
        self.start_pos = start_pos
        self.word = '__EOS__'
        self.yomi = '__EOS__'
        self.prev = None
        self.cost = 0

    def is_bos(self):
        return False

    def is_eos(self):
        return True

    def get_key(self):
        return '__EOS__'  # わざと使わない。__EOS__ 考慮すると変換精度が落ちるので。。今は使わない。
        # うまく使えることが確認できれば、__EOS__/__EOS__ にする。

    def surface(self, evaluator: tinylisp.Evaluator):
        return '__EOS__'

    def __repr__(self):
        return f"<EosNode: start_pos={self.start_pos}, prev={self.prev.word if self.prev else '-'}>"

    def calc_node_cost(self, user_language_model, system_language_model):
        return 0


class Node(AbstractNode):
    cost: Optional[float]

    def __init__(self, start_pos, word, yomi):
        super().__init__()
        if len(word) == 0:
            raise AssertionError(f"len(word) should not be 0")

        self.start_pos = start_pos
        self.word = word
        self.yomi = yomi
        self.prev = None
        self.cost = 0
        self._key = f"{self.word}/{self.yomi}"

    def __repr__(self):
        return f"<Node: start_pos={self.start_pos}, word={self.word}," \
               f" cost={self.cost}, prev={self.prev.word if self.prev else '-'} yomi={self.yomi}>"

    def is_bos(self):
        return False

    def is_eos(self):
        return False

    def get_key(self) -> str:
        return self._key

    def __eq__(self, other):
        if other is None:
            return False
        return self.__dict__ == other.__dict__

    def surface(self, evaluator: tinylisp.Evaluator):
        if self.word.startswith('('):
            return evaluator.run(self.word)
        else:
            return self.word

    def calc_node_cost(self, user_language_model, system_language_model) -> float:
        key = self.get_key()
        u = user_language_model.get_unigram_cost(key)
        if u is not None:
            # self.logger.info(f"Use user score: {node.get_key()} -> {u}")
            return u
        # print(f"SYSTEM LANGUAGE MODEL UNIGRAM: {key}")
        word_id, score = system_language_model.find_unigram(key)
        self.id = word_id
        return score if word_id >= 0 else UNIGRAM_DEFAULT_COST
