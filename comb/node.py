class Node:
    cost: float

    def __init__(self, start_pos, word, yomi):
        self.start_pos = start_pos
        self.word = word
        self.yomi = yomi
        self.prev = None

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