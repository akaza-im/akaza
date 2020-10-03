from typing import Optional

from akaza import tinylisp


class AbstractNode:
    id: Optional[int]

    def __init__(self):
        self._bigram_cache = {}
        self.id = None

    def is_eos(self):
        raise NotImplemented()

    def is_bos(self):
        raise NotImplemented()

    def get_bigram_cost(self, language_model, next_node):
        next_node_key = next_node.get_key()
        if next_node_key in self._bigram_cache:
            return self._bigram_cache[next_node_key]
        else:
            cost = language_model.calc_bigram_cost(self, next_node)
            self._bigram_cache[next_node_key] = cost
            return cost


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
