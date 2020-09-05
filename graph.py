import sys
from typing import Dict, List

import marisa_trie
from comb import SystemDict


class Node:
    def __init__(self, start_pos, word):
        self.start_pos = start_pos
        self.word = word
        self.cost = -len(word)
        self.prev = None

    def __repr__(self):
        return f"<Node: start_pos={self.start_pos}, word={self.word}, cost={self.cost}, prev={self.prev.word if self.prev else '-'}>"

    def get_cost(self):
        return - len(self.word)

    def is_bos(self):
        return self.word == '<S>'

    def is_eos(self):
        return self.word == '</S>'


def create_graph(size: int) -> Dict[int, List[Node]]:
    return {
        0: [Node(start_pos=-9999, word='<S>')],
        size: [Node(start_pos=-2, word='</S>')],
    }


def lookup(s, d: SystemDict):
    for i in range(0, len(s)):
        substr = s[i:]
        words = d.trie.prefixes(substr)
        for word in words:
            print(f"+-+-+-= {d.trie[word]}")
            yield word, d.trie[word][0].decode('utf-8').split('/') + [word]


# n文字目でおわる単語リストを作成する
def graph_construct(s, ht):
    graph = create_graph(len(s))
    print(graph)

    for i in range(0, len(s)):
        for j in range(i + 1, min(len(s) + 1, 16)):
            substr = s[i:j]
            if substr in ht:
                for word in ht[substr]:
                    node = Node(i, word)
                    if j not in graph:
                        graph[j] = []
                    # print(f"graph[{j}]={graph[j]} graph={graph}")
                    graph[j].append(node)

    return graph


def node_list(graph, i):
    return graph[i]


def get_prev_node(graph, node: Node) -> List[Node]:
    if node.is_eos():
        return graph[len(graph) - 1]
    else:
        return graph[node.start_pos]


# TODO: エッジコスト的なももも考慮されたい。

def viterbi(graph):
    print("Viterbi phase 1")
    for i in range(1, len(graph)):
        nodes = node_list(graph, i)
        for node in nodes:
            node_cost = node.get_cost()
            cost = sys.maxsize
            shortest_prev = None
            prev_nodes = get_prev_node(graph, node)
            if prev_nodes[0].is_bos():
                pass
            else:
                for prev_node in prev_nodes:
                    tmp_cost = prev_node.cost + node_cost
                    if tmp_cost < cost:
                        cost = tmp_cost
                        shortest_prev = prev_node
                node.prev = shortest_prev
                node.cost = cost

    dump_graph(graph)
    # sys.exit(1) # XXXXXXXXXXXXXXXX

    print("Viterbi phase 2")
    node = graph[len(graph) - 1][0]
    result = []
    while not node.is_bos():
        result.append(node)
        node = node.prev
    return reversed(result)


def dump_graph(graph):
    with open('hello.dot', 'w') as fp:
        fp.write("""digraph graph_name {\n""")
        fp.write("""  graph [\n""")
        fp.write("""    charset="utf-8"\n""")
        fp.write("""  ]\n""")
        for i, nodes in graph.items():
            for node in nodes:
                if node.is_eos():
                    fp.write(f"""  {i} -> {i} [label="{node.word}"]\n""")
                else:
                    fp.write(f"""  {node.start_pos} -> {i} [label="{node.word}"]\n""")
        fp.write("""}\n""")


# TODO: generate diagram via graphviz...
def main():
    src = 'きょうは'
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
    graph = graph_construct(src, ht)
    print(graph)
    print(viterbi(graph))


main()
