import logging
from tempfile import TemporaryDirectory
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from pyakaza.bind import Akaza, GraphResolver, BinaryDict, SystemUnigramLM, SystemBigramLM, Node, UserLanguageModel, \
    Slice

system_unigram_lm = SystemUnigramLM()
system_unigram_lm.load("../akaza-data/akaza_data/data/lm_v2_1gram.trie")

system_bigram_lm = SystemBigramLM()
system_bigram_lm.load("../akaza-data/akaza_data/data/lm_v2_2gram.trie")

tmpdir = TemporaryDirectory()
user_language_model = UserLanguageModel(
    tmpdir.name + "/uni",
    tmpdir.name + "/bi"
)

system_dict = BinaryDict()
system_dict.load("../akaza-data/akaza_data/data/system_dict.trie")

single_term = BinaryDict()
single_term.load("../akaza-data/akaza_data/data/single_term.trie")

logging.basicConfig(level=logging.DEBUG)


def test_wnn():
    src = 'わたしのなまえはなかのです'
    expected = '私の名前は中野です'

    resolver = GraphResolver(
        user_language_model,
        system_unigram_lm,
        system_bigram_lm,
        [system_dict],
        [single_term],
    )
    graph = resolver.graph_construct(src, None)

    resolver.fill_cost(graph)
    graph.dump()
    clauses = resolver.find_nbest(graph)
    got = ''.join([clause[0].get_word() for clause in clauses])

    # print(graph)

    assert got == expected


def test_graph_extend():
    src = 'はなか'
    resolver = GraphResolver(
        user_language_model,
        system_unigram_lm,
        system_bigram_lm,
        [system_dict],
        [single_term],
    )
    # (0,2) の文節を強制指定する
    graph = resolver.graph_construct(src, [
        Slice(0, 2),
        Slice(2, 3)
    ])
    # assert 1 not in graph.d
    # TODO

    # def test_katakana_candidates():
    #     src = 'ひょいー'
    #     resolver = GraphResolver(
    #         user_language_model=user_language_model,
    #         system_unigram_lm=system_unigram_lm,
    #         system_bigram_lm=system_bigram_lm,
    #         normal_dicts=[system_dict],
    #         single_term_dicts=[single_term],
    #     )
    #     graph = resolver.graph_construct(src, [
    #         Slice(0, 4)
    #     ])
    #     print(graph)
    #
    #     resolver.fill_cost(graph)
    #     clauses = resolver.find_nbest(graph)
    #     print(clauses)
    #     got = '/'.join([node.word for node in clauses[0]])
    #
    #     assert got == 'ひょいー/ヒョイー/hyoiー/ｈｙｏｉー'


# 「ひょいー」のような辞書に登録されていない単語に対して、カタカナ候補を提供すべき。
def test_katakana_candidates_for_unknown_word():
    # ユーザー言語モデルで「ヒョイー」のコストを高めておく。
    my_tmpdir = TemporaryDirectory()
    my_user_language_model = UserLanguageModel(
        my_tmpdir.name + "/uni",
        my_tmpdir.name + "/bi"
    )
    my_user_language_model.add_entry(
        [Node(0, 'ひょいー', 'ヒョイー')]
    )
    my_user_language_model.add_entry(
        [Node(0, 'ひょいー', 'ヒョイー')]
    )
    my_user_language_model.add_entry(
        [Node(0, 'ひょいー', 'ヒョイー')]
    )
    my_user_language_model.add_entry(
        [Node(0, 'ひょいー', 'ヒョイー')]
    )

    src = 'ひょいー'

    print(my_user_language_model.has_unigram_cost_by_yomi('ひょいー'))

    resolver = GraphResolver(
        my_user_language_model,
        system_unigram_lm,
        system_bigram_lm,
        [system_dict],
        [single_term],
    )
    graph = resolver.graph_construct(src, None)
    assert 'ひょいー' in set([node.get_yomi() for node in graph.get_items()])
    # print(graph)

    resolver.fill_cost(graph)
    clauses = resolver.find_nbest(graph)
    # print(graph.d[4])
    got = '/'.join([node.get_word() for node in clauses[0]])

    # ユーザー言語もでるにもといづいて、ヒョイーのスコアがあがって上位にでるようになっている。
    assert got == 'ヒョイー/ひょいー'
