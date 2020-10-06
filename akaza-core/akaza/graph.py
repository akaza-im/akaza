import logging
from logging import Logger
from typing import Dict, List

from akaza.node import BosNode, EosNode, AbstractNode


class Graph:
    logger: Logger
    d: Dict[int, List[AbstractNode]]

    def __init__(self, size: int, logger=logging.getLogger(__name__)):
        self.d = {
            0: [BosNode()],
            size + 1: [EosNode(start_pos=size)],
        }
        self.logger = logger

    def __len__(self) -> int:
        return len(self.d)

    def __repr__(self) -> str:
        s = ''
        for i in sorted(self.d.keys()):
            if i in self.d:
                s += f"{i}:\n"
                s += "\n".join(["\t" + str(x) for x in sorted(self.d[i], key=lambda x: x.cost)]) + "\n"
        return s

    def append(self, index: int, node: AbstractNode) -> None:
        if index not in self.d:
            self.d[index] = []
        # print(f"graph[{j}]={graph[j]} graph={graph}")
        self.d[index].append(node)

    def get_items(self):
        for i in sorted(self.d.keys()):
            if i == 0:  # skip bos
                continue
            yield self.d[i]

    def all_nodes(self):
        for i in sorted(self.d.keys()):
            nodes = self.d[i]
            for node in nodes:
                if node.is_eos() or node.is_bos():
                    continue
                yield node

    def get_item(self, i: int) -> List[AbstractNode]:
        return self.d[i]

    def dump(self, path: str):
        with open(path, 'w') as fp:
            fp.write("""digraph graph_name {\n""")
            fp.write("""  graph [\n""")
            fp.write("""    charset="utf-8"\n""")
            fp.write("""  ]\n""")
            for i, nodes in self.d.items():
                for node in nodes:
                    fp.write(f"  {node.start_pos} -> {i} [label=\"{node.word}:"
                             f" {node.cost}: {node.prev.word if node.prev else '-'}\"]\n")
            fp.write("""}\n""")

    def get_eos(self):
        return self.d[max(self.d.keys())][0]

    def get_bos(self):
        return self.d[0][0]
