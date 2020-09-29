from typing import Optional

from akaza import tinylisp


class AbstractNode:
    def is_eos(self):
        raise NotImplemented()

    def is_bos(self):
        raise NotImplemented()


class BosNode(AbstractNode):
    def __init__(self):
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


class EosNode(AbstractNode):
    def __init__(self, start_pos):
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


class Node(AbstractNode):
    cost: Optional[float]

    def __init__(self, start_pos, word, yomi):
        if len(word) == 0:
            raise AssertionError(f"len(word) should not be 0")

        self.start_pos = start_pos
        self.word = word
        self.yomi = yomi
        self.prev = None
        self.cost = 0

    def __repr__(self):
        return f"<Node: start_pos={self.start_pos}, word={self.word}," \
               f" cost={self.cost}, prev={self.prev.word if self.prev else '-'} yomi={self.yomi}>"

    def is_bos(self):
        return False

    def is_eos(self):
        return False

    def get_key(self) -> str:
        return f"{self.word}/{self.yomi}"

    def __eq__(self, other):
        if other is None:
            return False
        return self.__dict__ == other.__dict__

    def __hash__(self):
        # necessary for instances to behave sanely in dicts and sets.
        return hash((self.start_pos, self.word, self.yomi))

    def surface(self, evaluator: tinylisp.Evaluator):
        if self.word.startswith('('):
            return evaluator.run(self.word)
        else:
            return self.word
