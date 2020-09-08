import sys
from typing import Dict, List
import marisa_trie
import math
import logging
import jaconv

from comb.system_dict import SystemDict

DEFAULT_SCORE = [(math.log10(0.00000000001),)]


class Node:
    cost: float

    def __init__(self, start_pos, word, yomi, unigram_score, bigram_score):
        self.start_pos = start_pos
        self.word = word
        self.yomi = yomi
        self.unigram_score = unigram_score
        self.bigram_score = bigram_score
        self.cost = self.calc_node_cost()
        self.prev = None

    def __repr__(self):
        return f"<Node: start_pos={self.start_pos}, word={self.word}," \
               f" cost={self.cost}, prev={self.prev.word if self.prev else '-'}>"

    def calc_node_cost(self) -> float:
        if self.is_bos():
            return 0
        elif self.is_eos():
            return 0
        else:
            return self.unigram_score.get(self.get_key(), DEFAULT_SCORE)[0][0]

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
            return f"{self.yomi}/{self.word}"

    def calc_bigram_cost(self, node) -> float:
        # self → node で処理する。
        return self.bigram_score.get(f"{self.get_key()}\t{node.get_key()}", DEFAULT_SCORE)[0][0]


class Graph:
    d: Dict[int, List[Node]]

    def __init__(self, size: int, unigram_score, bigram_score):
        self.d = {
            0: [Node(start_pos=-9999, word='<S>', yomi='<S>', unigram_score=unigram_score,
                     bigram_score=bigram_score)],
            size + 1: [
                Node(start_pos=size, word='</S>', yomi='</S>', unigram_score=unigram_score,
                     bigram_score=bigram_score)],
        }

    def __len__(self) -> int:
        return len(self.d)

    def __repr__(self) -> str:
        s = ''
        for i in sorted(self.d.keys()):
            if i in self.d:
                s += f"{i}:\n"
                s += "\n".join(["\t" + str(x) for x in self.d[i]]) + "\n"
        return s

    def append(self, index: int, node: Node) -> None:
        if index not in self.d:
            self.d[index] = []
        # print(f"graph[{j}]={graph[j]} graph={graph}")
        self.d[index].append(node)

    def __getitem__(self, item):
        ary = [None for _ in range(len(self.d))]
        try:
            for k in sorted(self.d.keys()):
                ary[k] = self.d[k]
        except IndexError:
            logging.error(f"Cannot get entry {self.d[15]} {k}", exc_info=True)
            sys.exit(1)
        return ary[item]

    def dump(self, path: str):
        with open(path, 'w') as fp:
            fp.write("""digraph graph_name {\n""")
            fp.write("""  graph [\n""")
            fp.write("""    charset="utf-8"\n""")
            fp.write("""  ]\n""")
            for i, nodes in self.d.items():
                for node in nodes:
                    fp.write(f"  {node.start_pos} -> {i} [label=\"{node.word}:"
                             f" {node.cost}: node={node.calc_node_cost()} {node.prev.word if node.prev else '-'}\"]\n")
            fp.write("""}\n""")


def lookup(s, system_dict: SystemDict):
    for i in range(0, len(s)):
        yomi = s[i:]
        # print(f"YOMI:::: {yomi}")
        words = system_dict.trie.prefixes(yomi)
        if len(words) > 0:
            # print(f"YOMI:::: {yomi} {words}")
            for word in words:
                kanjis = system_dict.trie[word][0].decode('utf-8').split('/')
                yield word, (kanjis + [word, jaconv.hira2kata(word)])
        else:
            # print(f"YOMI~~~~:::: {yomi}")
            yield yomi[0], [yomi[0], jaconv.hira2kata(yomi[0])]


# n文字目でおわる単語リストを作成する
def graph_construct(s, ht, unigram_score, bigram_score):
    graph = Graph(size=len(s), unigram_score=unigram_score, bigram_score=bigram_score)

    for i in range(0, len(s)):
        for j in range(i + 1, len(s) + 1):
            # substr は「読み」であろう。
            # word は「漢字」であろう。
            yomi = s[i:j]
            if yomi in ht:
                # print(f"YOMI YOMI: {yomi} {ht[yomi]}")
                for kanji in ht[yomi]:
                    node = Node(i, kanji, yomi, unigram_score=unigram_score, bigram_score=bigram_score)
                    graph.append(index=j, node=node)
            else:
                # print(f"NO YOMI: {yomi}")
                pass
                # graph.append(j, Node(j, yomi, yomi, unigram_score=unigram_score, bigram_score=bigram_score))

    return graph


def get_prev_node(graph, node: Node) -> List[Node]:
    return graph[node.start_pos]


def viterbi(graph: Graph, onegram_trie):
    print("Viterbi phase 1")
    for nodes in graph[1:]:
        # print(f"fFFFF {nodes}")
        for node in nodes:
            # print(f"  PPPP {node}")
            node_cost = node.calc_node_cost()
            # print(f"  NC {node.word} {node_cost}")
            cost = -sys.maxsize
            shortest_prev = None
            prev_nodes = get_prev_node(graph, node)
            if prev_nodes[0].is_bos():
                node.prev = prev_nodes[0]
                node.cost = node_cost
            else:
                for prev_node in prev_nodes:
                    if prev_node.cost is None:
                        logging.error(f"Missing prev_node.cost: {prev_node}")
                    tmp_cost = prev_node.cost + prev_node.calc_bigram_cost(node) + node_cost
                    if cost < tmp_cost:
                        cost = tmp_cost
                        shortest_prev = prev_node
                # print(f"    SSSHORTEST: {shortest_prev} in {prev_nodes}")
                node.prev = shortest_prev
                node.cost = cost

    print("Viterbi phase 2")
    node = graph[len(graph) - 1][0]
    # print(node)
    result = []
    while not node.is_bos():
        result.append(node)
        if node == node.prev:
            raise AssertionError(f"node==node.prev: {node}")
        node = node.prev
    return list(reversed(result))


# TODO: generate diagram via graphviz...
def main():
    logging.basicConfig(level=logging.DEBUG)

    # src = 'きょうはいいてんきですね'
    # src = 'きょうは'
    # src = 'きょうのてんきは'
    # src = 'わたしのなまえはなかのです'
    # src = 'すももももももももものうち'
    # src = 'せいきゅう'
    # src = 'しはらいにちじ'
    # src = 'せいきゅうしょのしはらいにちじ'

    unigram_score = marisa_trie.RecordTrie('@f')
    unigram_score.load('model/jawiki.1gram')

    bigram_score = marisa_trie.RecordTrie('@f')
    bigram_score.load('model/jawiki.2gram')
    system_dict = SystemDict()

    # print(ht)
    def run(src):
        if True:
            ht = dict(lookup(src, system_dict))
        else:
            ht = {
                'き': ['木', '気', 'き'],
                'きょ': ['虚', 'きょ'],
                'きょう': ['今日', 'きょう'],
                'ょ': ['ょ'],
                'ょう': ['ょう'],
                'う': ['う'],
                'は': ['葉', 'は'],
                'うは': ['右派', 'うは'],
                'せ': ['せ'],
                'い': ['い'],
                'き': ['き'],
                'ゅ': ['ゅ'],
            }
        graph = graph_construct(src, ht, unigram_score, bigram_score)

        got = viterbi(graph, unigram_score)
        # print(graph)
        print(' '.join([f"<{x.yomi}/{x.word}>" for x in got if not x.is_eos()]))

    # http://cl.sd.tmu.ac.jp/~komachi/chaime/index.html
    run('わたしのなまえはなかのです')
    run('しはらいにちじ')
    run('えんとりーすう')
    run('せいきゅうしょのしはらいにちじ')
    run('ちかくしじょうちょうさをおこなう')
    dat = [
        ('せいきゅうしょのしはらいにちじ', '請求書の支払い日時'),
        ('ちかくしじょうちょうさをおこなう。', '近く市場調査を行う。'),
        ('そのごさいとないで', 'その後サイト内で'),
        ('きょねんにくらべたかいすいじゅんだ。', '去年に比べ高い水準だ。'),
        ('ひるいちまでにしょるいつくっといて', '昼イチまでに書類作っといて。'),
        ('そんなはなししんじっこないよね。', 'そんな話信じっこないよね。'),
        ('はじめっからもってけばいいのに。', '初めっからもってけばいいのに。'),
        ('あつあつのにくまんにぱくついた。', '熱々の肉まんにぱくついた。'),
    ]
    for kana, kanji in dat:
        run(kana)
        print(f"Expected: {kanji}")


#    for ww in ["しはらい/支払い", "きょう/橋", "きょう/今日", "きょう/頃", "きょう/きょう"]:
#        print(f"WWWWW {ww} {unigram_score.get(ww, DEFAULT_SCORE)}")
#    for ww in ["きょう/橋\tは/は", "きょう/今日\tは/は", "きょう/頃\tは/は", "は/は\tきょう/今日", "は/は\tきょう/頃"]:
#        print(f"WWWWW {ww} {bigram_score.get(ww, DEFAULT_SCORE)}")

if __name__ == '__main__':
    main()
