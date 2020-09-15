import logging
from tempfile import TemporaryDirectory
import sys
import pathlib

sys.path.append(str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

import pytest
from akaza.graph import lookup, graph_construct, viterbi
from akaza.language_model import LanguageModel
from akaza.user_language_model import UserLanguageModel
from akaza_data.system_dict import SystemDict
from akaza_data.system_language_model import SystemLanguageModel

system_language_model = SystemLanguageModel.load()

tmpdir = TemporaryDirectory()
user_language_model = UserLanguageModel(tmpdir.name)

language_model = LanguageModel(system_language_model, user_language_model=user_language_model)

system_dict = SystemDict.load()

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
    # ('そうみたいですね', 'そうみたいですね'),
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
