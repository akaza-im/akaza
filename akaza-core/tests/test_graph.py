import logging
from tempfile import TemporaryDirectory
import sys
import pathlib

sys.path.append(str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

import pytest
from akaza.dictionary import Dictionary
from akaza.node import Node
from akaza.graph import GraphResolver
from akaza.language_model import LanguageModel
from akaza.user_language_model import UserLanguageModel
from akaza_data.system_dict import SystemDict
from akaza_data.system_language_model import SystemLanguageModel

system_language_model = SystemLanguageModel.load()

tmpdir = TemporaryDirectory()
user_language_model = UserLanguageModel(tmpdir.name)

language_model = LanguageModel(system_language_model, user_language_model=user_language_model)

system_dict = SystemDict.load()
dictionary = Dictionary(
    system_dict=system_dict,
    user_dicts=[],
)

logging.basicConfig(level=logging.DEBUG)


@pytest.mark.parametrize('src, expected', [
    # Wnn で有名なフレーズ。
    ('わたしのなまえはなかのです', '私の名前は中野です'),
    # カタカナ語の処理が出来ていること。
    ('わーど', 'ワード'),
    ('にほん', '日本'),
    ('ややこしい', 'ややこしい'),
    ('むずかしくない', '難しくない'),
    ('きぞん', '既存'),
    ('のぞましい', '望ましい'),
    ('こういう', 'こういう'),
    ('はやくち', '早口'),
    # ('どっぐふーでぃんぐしづらい', 'ドッグフーディング仕辛い'),
    ('しょうがっこう', '小学校'),
    ('げすとだけ', 'ゲストだけ'),
    ('ぜんぶでてるやつ', '全部でてるやつ'),
    ('えらべる', '選べる'),
    ('わたしだよ', '私だよ'),
    # ('にほんごじょうほう', '日本語情報'),
    # ('そうみたいですね', 'そうみたいですね'),
    ('きめつのやいば', '鬼滅の刃'),
    ('れいわ', '令和'),
])
def test_expected(src, expected):
    resolver = GraphResolver(language_model=language_model, dictionary=dictionary)

    ht = dict(resolver.lookup(src))
    graph = resolver.graph_construct(src, ht)

    clauses = resolver.viterbi(graph)
    print(graph)
    got = ''.join([clause[0].word for clause in clauses])

    assert got == expected


def test_wnn():
    src = 'わたしのなまえはなかのです'
    expected = '私の名前は中野です'

    resolver = GraphResolver(language_model=language_model, dictionary=dictionary)
    ht = dict(resolver.lookup(src))
    graph = resolver.graph_construct(src, ht)

    clauses = resolver.viterbi(graph)
    got = ''.join([clause[0].word for clause in clauses])

    # print(graph)

    assert got == expected


def test_graph_extend():
    src = 'はなか'
    resolver = GraphResolver(language_model=language_model, dictionary=dictionary)
    ht = dict(resolver.lookup(src))
    # (0,2) の文節を強制指定する
    graph = resolver.graph_construct(src, ht, [
        slice(0, 2),
        slice(2, 3)
    ])
    assert 1 not in graph.d


# 「ひょいー」のような辞書に登録されていない単語に対して、カタカナ候補を提供すべき。
def test_katakana_candidates():
    src = 'ひょいー'
    resolver = GraphResolver(language_model=language_model, dictionary=dictionary)
    ht = dict(resolver.lookup(src))
    for k, v in ht.items():
        print(f"{k}:{v}")
    graph = resolver.graph_construct(src, ht, [
        slice(0, 4)
    ])
    print(graph)

    clauses = resolver.viterbi(graph)
    print(clauses)
    got = '/'.join([node.word for node in clauses[0]])

    assert got == 'ひょいー/ヒョイー/hyoiー/ｈｙｏｉー'


# 「ひょいー」のような辞書に登録されていない単語に対して、カタカナ候補を提供すべき。
def test_katakana_candidates_for_unknown_word():
    # ユーザー言語モデルで「ヒョイー」のコストを高めておく。
    my_tmpdir = TemporaryDirectory()
    my_user_language_model = UserLanguageModel(my_tmpdir.name)
    my_user_language_model.add_entry(
        [Node(start_pos=0, word='ヒョイー', yomi='ひょいー')]
    )
    my_user_language_model.add_entry(
        [Node(start_pos=0, word='ヒョイー', yomi='ひょいー')]
    )
    my_user_language_model.add_entry(
        [Node(start_pos=0, word='ヒョイー', yomi='ひょいー')]
    )
    my_user_language_model.add_entry(
        [Node(start_pos=0, word='ヒョイー', yomi='ひょいー')]
    )

    src = 'ひょいー'

    my_language_model = LanguageModel(
        system_language_model=system_language_model,
        user_language_model=my_user_language_model
    )
    print(my_user_language_model.has_unigram_cost_by_yomi('ひょいー'))
    print(my_language_model.has_unigram_cost_by_yomi('ひょいー'))

    resolver = GraphResolver(language_model=my_language_model, dictionary=dictionary)
    ht = dict(resolver.lookup(src))
    graph = resolver.graph_construct(src, ht)
    assert 'ひょいー' in set([node.yomi for node in graph.all_nodes()])
    # print(graph)

    clauses = resolver.viterbi(graph)
    print(graph.d[4])
    got = '/'.join([node.word for node in clauses[0]])

    # ユーザー言語もでるにもといづいて、ヒョイーのスコアがあがって上位にでるようになっている。
    assert got == 'ヒョイー/ひょいー'
