import sys
from typing import List

import jaconv

from akaza.dictionary import Dictionary
from akaza.graph import Graph
from akaza.language_model import LanguageModel
from akaza.node import Node, AbstractNode
from akaza_data.systemlm_loader import BinaryDict


class GraphResolver:
    def __init__(self,
                 language_model: LanguageModel,
                 dictionary: Dictionary,
                 single_term_dicts: List[BinaryDict]):
        self.dictionary = dictionary
        self.language_model = language_model
        self.single_term_dicts = single_term_dicts

    def lookup(self, s: str):
        assert self.language_model

        for i in range(0, len(s)):
            yomi = s[i:]
            # print(f"YOMI:::: {yomi}")
            words = self.dictionary.prefixes(yomi)
            if len(words) > 0:
                # print(f"YOMI:::: {yomi} {words}")
                for word in words:
                    kanjis = self.dictionary[word]
                    if word not in kanjis:
                        kanjis.append(word)

                    kata = jaconv.hira2kata(word)
                    if kata not in kanjis:
                        kanjis.append(kata)

                    if word == yomi:
                        for single_term_dict in self.single_term_dicts:
                            for emoji in single_term_dict.find_kanjis(yomi):
                                if emoji not in kanjis:
                                    kanjis.append(emoji)

                    yield word, kanjis

                if yomi not in words and self.language_model.has_unigram_cost_by_yomi(yomi):
                    # システム辞書に入ってないがユーザー言語モデルには入っているという場合は候補にいれる。
                    kanjis = [yomi]

                    kata = jaconv.hira2kata(yomi)
                    if kata not in kanjis:
                        kanjis.append(kata)
                    for single_term_dict in self.single_term_dicts:
                        for emoji in single_term_dict.find_kanjis(yomi):
                            if emoji not in kanjis:
                                kanjis.append(emoji)

                    yield yomi, kanjis
            else:
                # print(f"YOMI~~~~:::: {yomi}")
                kanjis = [yomi[0]]
                hira = jaconv.hira2kata(yomi[0])
                if hira not in kanjis:
                    kanjis.append(hira)
                for single_term_dict in self.single_term_dicts:
                    for emoji in single_term_dict.find_kanjis(yomi):
                        if emoji not in kanjis:
                            kanjis.append(emoji)
                yield yomi[0], kanjis

    # n文字目でおわる単語リストを作成する
    def graph_construct(self, s, ht, force_selected_clause: List[slice] = None) -> Graph:
        graph = Graph(size=len(s))

        if force_selected_clause:
            for force_slice in force_selected_clause:
                # 強制的に範囲を指定されている場合。
                # substr は「読み」であろう。
                # word は「漢字」であろう。
                yomi = s[force_slice]
                i = force_slice.start
                j = force_slice.stop
                # print(f"XXXX={s} {force_slice} {yomi}")
                if yomi in ht:
                    # print(f"YOMI YOMI: {yomi} {ht[yomi]}")
                    for kanji in ht[yomi]:
                        node = Node(i, kanji, yomi)
                        graph.append(index=j, node=node)
                else:
                    # print(f"NO YOMI: {yomi}")
                    if len(yomi) == 0:
                        raise AssertionError(f"len(yomi) should not be 0. {s}, {force_slice}")
                    for word in [yomi, jaconv.hira2kata(yomi), jaconv.kana2alphabet(yomi),
                                 jaconv.h2z(jaconv.kana2alphabet(yomi), ascii=True)]:
                        node = Node(start_pos=i, word=word, yomi=yomi)
                        graph.append(index=j, node=node)
        else:
            for i in range(0, len(s)):
                # print(f"LOOP {i}")
                # for i in range(0, len(s)):
                for j in range(i + 1, len(s) + 1):
                    # substr は「読み」であろう。
                    # word は「漢字」であろう。
                    yomi = s[i:j]
                    if yomi in ht:
                        # print(f"YOMI YOMI: {yomi} {ht[yomi]}")
                        for kanji in ht[yomi]:
                            node = Node(i, kanji, yomi)
                            graph.append(index=j, node=node)
                    else:
                        if self.language_model.has_unigram_cost_by_yomi(yomi):
                            for word in [yomi, jaconv.hira2kata(yomi), jaconv.kana2alphabet(yomi),
                                         jaconv.h2z(jaconv.kana2alphabet(yomi), ascii=True)]:
                                node = Node(start_pos=i, word=word, yomi=yomi)
                                graph.append(index=j, node=node)

        return graph

    # @profile
    def viterbi(self, graph: Graph) -> List[List[AbstractNode]]:
        """
        ビタビアルゴリズムにもとづき、最短の経路を求めて、N-Best 解を求める。
        """

        self.fill_cost(graph)
        return self.find_nbest(graph)

    # @profile
    def fill_cost(self, graph: Graph):
        """
        Graph の各ノードについて最短のノードをえる。
        """
        # BOS にスコアを設定。
        graph.get_bos().cost = 0

        for nodes in graph.get_items():
            # print(f"fFFFF {nodes}")
            for node in nodes:
                # print(f"  PPPP {node}")
                node_cost = self.language_model.calc_node_cost(node)
                # print(f"  NC {node.word} {node_cost}")
                cost = -sys.maxsize
                shortest_prev = None
                prev_nodes = graph.get_item(node.start_pos)
                if prev_nodes[0].is_bos():
                    node.prev = prev_nodes[0]
                    node.cost = node_cost
                else:
                    for prev_node in prev_nodes:
                        bigram_cost = prev_node.get_bigram_cost(self.language_model, node)
                        tmp_cost = prev_node.cost + bigram_cost + node_cost
                        if cost < tmp_cost:  # コストが最大になる経路をえらんでいる。
                            cost = tmp_cost
                            shortest_prev = prev_node
                    # print(f"    SSSHORTEST: {shortest_prev} in {prev_nodes}")
                    node.prev = shortest_prev
                    node.cost = cost

    # @profile
    def find_nbest(self, graph: Graph):
        # find EOS.
        node = graph.get_eos()

        result = []
        last_node = None
        while not node.is_bos():
            if node == node.prev:
                print(graph)
                raise AssertionError(f"node==node.prev: {node}")

            if not node.is_eos():
                # 他の候補を追加する。
                nodes = sorted(
                    [n for n in graph.get_item(node.start_pos + len(node.yomi)) if node.yomi == n.yomi],
                    key=lambda x: x.cost + x.get_bigram_cost(self.language_model, last_node), reverse=True)
                result.append(nodes)

            last_node = node
            node = node.prev
        return list(reversed(result))
