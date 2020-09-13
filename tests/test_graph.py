from tempfile import TemporaryDirectory

from comb.combromkan import to_hiragana
import pytest
import marisa_trie
from comb.system_dict import SystemDict
from comb.graph import lookup, graph_construct, viterbi
from comb.language_model import LanguageModel
import logging

from comb.system_language_model import SystemLanguageModel
from comb.user_language_model import UserLanguageModel

unigram_score = marisa_trie.RecordTrie('@f')
unigram_score.load('model/jawiki.1gram')

bigram_score = marisa_trie.RecordTrie('@f')
bigram_score.load('model/jawiki.2gram')

system_language_model = SystemLanguageModel(unigram_score, bigram_score)

tmpdir = TemporaryDirectory()
user_language_model = UserLanguageModel(tmpdir.name)

language_model = LanguageModel(system_language_model, user_language_model=user_language_model)

system_dict = SystemDict('model/system_dict.trie')

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
    ('ぜんぶでてるやつ', '全部でてる奴'),
    ('えらべる', '選べる'),
    ('そうみたいですね', 'そうみたいですね'),
    # ('きめつのやいば', '鬼滅の刃'),
    #    ('れいわ', '令和'),
])
def test_wnn(src, expected):
    ht = dict(lookup(src, system_dict, user_language_model, user_dict=None))
    graph = graph_construct(src, ht)

    clauses = viterbi(graph, language_model)
    got = ''.join([clause[0].word for clause in clauses])

    assert got == expected


def test_graph_extend():
    src = 'はなか'
    ht = dict(lookup(src, system_dict, user_language_model, user_dict=None))
    # (0,2) の文節を強制指定する
    graph = graph_construct(src, ht, [
        slice(0, 2),
        slice(2, 3)
    ])
    assert 1 not in graph.d
