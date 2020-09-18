from typing import Optional


class Node:
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
        return self.word == '<S>'

    def is_eos(self):
        return self.word == '</S>'

    def get_key(self) -> str:
        if self.is_bos():
            return '<S>'
        elif self.is_eos():
            return '</S>'
        else:
            return f"{self.word}/{self.yomi}"

    def __eq__(self, other):
        return self.__dict__ == other.__dict__

    def __hash__(self):
        # necessary for instances to behave sanely in dicts and sets.
        return hash((self.start_pos, self.word, self.yomi, self.prev, self.cost))
