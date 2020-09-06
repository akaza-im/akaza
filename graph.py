import sys
from typing import Dict, List
import marisa_trie

from comb import SystemDict


class Node:
    def __init__(self, start_pos, word, onegram_score):
        self.start_pos = start_pos
        self.word = word
        self.onegram_score = onegram_score
        self.cost = onegram_score.get(word, -0.001)
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
            m = self.onegram_score.get(self.word, -0.001)
            if type(m) == tuple:
                m = m[0]
            if type(m) == list:
                m = m[0]
            if type(m) == tuple:
                m = m[0]
            return m

    def is_bos(self):
        return self.word == '<S>'

    def is_eos(self):
        return self.word == '</S>'


class Graph:
    d: Dict[int, List[Node]]

    def __init__(self, size: int, onegram_score):
        self.d = {
            0: [Node(start_pos=-9999, word='<S>', onegram_score=onegram_score)],
            size + 1: [Node(start_pos=size, word='</S>', onegram_score=onegram_score)],
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

    def append(self, index, node):
        if index not in self.d:
            self.d[index] = []
        # print(f"graph[{j}]={graph[j]} graph={graph}")
        self.d[index].append(node)

    def __getitem__(self, item):
        ary = [None for _ in range(len(self.d))]
        for k in sorted(self.d.keys()):
            ary[k] = self.d[k]
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
                             f" {node.cost}: {node.calc_node_cost()}\"]\n")
            fp.write("""}\n""")


def lookup(s, d: SystemDict):
    for i in range(0, len(s)):
        substr = s[i:]
        words = d.trie.prefixes(substr)
        for word in words:
            print(f"+-+-+-= {d.trie[word]}")
            yield word, d.trie[word][0].decode('utf-8').split('/') + [word]


# n文字目でおわる単語リストを作成する
def graph_construct(s, ht, onegram_score):
    graph = Graph(size=len(s), onegram_score=onegram_score)
    print(graph)
    print(len(graph))

    for i in range(0, len(s)):
        for j in range(i + 1, min(len(s) + 1, 16)):
            substr = s[i:j]
            if substr in ht:
                for word in ht[substr]:
                    node = Node(i, word, onegram_score=onegram_score)
                    graph.append(j, node)

    return graph


def get_prev_node(graph, node: Node) -> List[Node]:
    return graph[node.start_pos]


# TODO: エッジコスト的なモノも考慮されたい。

def viterbi(graph: Graph, onegram_trie):
    print("Viterbi phase 1")
    for nodes in graph[1:]:
        print(f"fFFFF {nodes}")
        for node in nodes:
            print(f"  PPPP {node}")
            node_cost = node.calc_node_cost()
            print(f"  NC {node.word} {node_cost}")
            cost = sys.maxsize
            shortest_prev = None
            prev_nodes = get_prev_node(graph, node)
            if prev_nodes[0].is_bos():
                node.prev = prev_nodes[0]
                node.cost = node_cost
            else:
                for prev_node in prev_nodes:
                    # この単純に引くカタチはおかしいんじゃねえのか?
                    # スコアがふっとぶわ。
                    tmp_cost = prev_node.cost + node_cost
                    if tmp_cost < cost:
                        cost = tmp_cost
                        shortest_prev = prev_node
                print(f"    SSSHORTEST: {shortest_prev} in {prev_nodes}")
                node.prev = shortest_prev
                node.cost = cost

    print(graph)
    graph.dump('hello.dot')

    print("Viterbi phase 2")
    node = graph[len(graph) - 1][0]
    print(node)
    result = []
    while not node.is_bos():
        result.append(node)
        if node == node.prev:
            raise AssertionError(f"node==node.prev: {node}")
        node = node.prev
    return list(reversed(result))


# TODO: generate diagram via graphviz...
def main():
    # src = 'きょうはいいてんきですね'
    src = 'きょうは'
    # src = 'わたしのなまえはなかのです'
    onegram_score = marisa_trie.RecordTrie('@f')
    onegram_score.load('model/jawiki.1gram')
    if False:
        system_dict = SystemDict()
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
        }
    print(ht)
    graph = graph_construct(src, ht, onegram_score)
    print(graph)
    got = viterbi(graph, onegram_score)
    print(' '.join([x.word for x in got if not x.is_eos()]))


main()
