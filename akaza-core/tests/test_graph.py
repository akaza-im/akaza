import logging
from tempfile import TemporaryDirectory
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

import pytest
from akaza.node import Node
from akaza.graph_resolver import GraphResolver
from akaza.user_language_model import UserLanguageModel
from akaza_data.systemlm_loader import BinaryDict, SystemUnigramLM, SystemBigramLM

system_unigram_lm = SystemUnigramLM()
system_unigram_lm.load("../akaza-data/akaza_data/data/lm_v2_1gram.trie")

system_bigram_lm = SystemBigramLM()
system_bigram_lm.load("../akaza-data/akaza_data/data/lm_v2_2gram.trie")


tmpdir = TemporaryDirectory()
user_language_model = UserLanguageModel(tmpdir.name)

system_dict = BinaryDict()
system_dict.load("../akaza-data/akaza_data/data/system_dict.trie")

single_term = BinaryDict()
single_term.load("../akaza-data/akaza_data/data/single_term.trie")

logging.basicConfig(level=logging.DEBUG)


@pytest.mark.parametrize('src, expected', [
    # Wnn で有名なフレーズ。
    ('わたしのなまえはなかのです', '私の名前は中野です'),
    # カタカナ語の処理が出来ていること。
    ('わーど', 'ワード'),
    ('にほん', '日本'),
    ('ややこしい', 'ややこしい'),
    ('むずかしくない', '難しく無い'),
    ('きぞん', '既存'),
    ('のぞましい', '望ましい'),
    ('こういう', 'こういう'),
    ('はやくち', '早口'),
    # ('どっぐふーでぃんぐしづらい', 'ドッグフーディング仕辛い'),
    ('しょうがっこう', '小学校'),
    ('げすとだけ', 'ゲストだけ'),
    ('ぜんぶでてるやつ', '全部でてるやつ'),
    ('えらべる', '選べる'),
    ('わたしだよ', 'わたしだよ'),
    # ('にほんごじょうほう', '日本語情報'),
    # ('そうみたいですね', 'そうみたいですね'),
    ('きめつのやいば', '鬼滅の刃'),
    ('れいわ', '令和'),
])
def test_expected(src, expected):
    resolver = GraphResolver(
        user_language_model=user_language_model,
        system_unigram_lm=system_unigram_lm,
        system_bigram_lm=system_bigram_lm,
        normal_dicts=[system_dict],
        single_term_dicts=[single_term],
    )

    ht = dict(resolver.lookup(src))
    graph = resolver.graph_construct(src, ht)

    clauses = resolver.viterbi(graph)
    print(graph)
    got = ''.join([clause[0].word for clause in clauses])

    assert got == expected


def test_wnn():
    src = 'わたしのなまえはなかのです'
    expected = '私の名前は中野です'

    resolver = GraphResolver(
        user_language_model=user_language_model,
        system_unigram_lm=system_unigram_lm,
        system_bigram_lm=system_bigram_lm,
        normal_dicts=[system_dict],
        single_term_dicts=[single_term],
    )
    ht = dict(resolver.lookup(src))
    graph = resolver.graph_construct(src, ht)

    clauses = resolver.viterbi(graph)
    got = ''.join([clause[0].word for clause in clauses])

    # print(graph)

    assert got == expected


def test_graph_extend():
    src = 'はなか'
    resolver = GraphResolver(
        user_language_model=user_language_model,
        system_unigram_lm=system_unigram_lm,
        system_bigram_lm=system_bigram_lm,
        normal_dicts=[system_dict],
        single_term_dicts=[single_term],
    )
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
    resolver = GraphResolver(
        user_language_model=user_language_model,
        system_unigram_lm=system_unigram_lm,
        system_bigram_lm=system_bigram_lm,
        normal_dicts=[system_dict],
        single_term_dicts=[single_term],
    )
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
def test_emoji_candidates():
    src = 'すし'
    resolver = GraphResolver(
        user_language_model=user_language_model,
        system_unigram_lm=system_unigram_lm,
        system_bigram_lm=system_bigram_lm,
        normal_dicts=[system_dict],
        single_term_dicts=[single_term],
    )
    ht = dict(resolver.lookup(src))
    for k, v in ht.items():
        print(f"{k}:{v}")
    graph = resolver.graph_construct(src, ht, [
    ])
    print(graph)

    clauses = resolver.viterbi(graph)
    print(clauses)
    got = '/'.join([node.word for node in clauses[0]])

    assert '🍣' in got


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

    print(my_user_language_model.has_unigram_cost_by_yomi('ひょいー'))

    resolver = GraphResolver(
        user_language_model=my_user_language_model,
        system_unigram_lm=system_unigram_lm,
        system_bigram_lm=system_bigram_lm,
        normal_dicts=[system_dict],
        single_term_dicts=[single_term],
    )
    ht = dict(resolver.lookup(src))
    graph = resolver.graph_construct(src, ht)
    assert 'ひょいー' in set([node.yomi for node in graph.all_nodes()])
    # print(graph)

    clauses = resolver.viterbi(graph)
    print(graph.d[4])
    got = '/'.join([node.word for node in clauses[0]])

    # ユーザー言語もでるにもといづいて、ヒョイーのスコアがあがって上位にでるようになっている。
    assert got == 'ヒョイー/ひょいー'
