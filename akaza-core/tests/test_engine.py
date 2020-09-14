from tempfile import NamedTemporaryFile

import os

import pytest

from akaza import Akaza
from akaza.system_dict import SystemDict
from akaza.user_language_model import UserLanguageModel
from akaza.system_language_model import SystemLanguageModel

tmpfile = NamedTemporaryFile(delete=False)
user_language_model = UserLanguageModel(tmpfile.name)
system_dict = SystemDict('../akaza-data/system_dict.trie')

system_language_model = SystemLanguageModel.create('../akaza-data/system_language_model.trie')

akaza = Akaza(
    user_language_model=user_language_model,
    system_dict=system_dict,
    user_dict=None,
    system_language_model=system_language_model
)


@pytest.mark.parametrize('src, expected', [
    # Wnn で有名なフレーズ。
    ('わたしのなまえはなかのです', '私の名前は中野です'),
    # カタカナ語の処理が出来ていること。
    ('わーど', 'ワード'),
    ('にほん', '日本'),
    ('それなwww', 'それなwww'),
    ('siinn', '子音'),
    ('zh', '←'),
    ('IME', 'IME'),
])
def test_wnn(src, expected):
    clauses = akaza.convert(src)
    got = ''.join([clause[0].word for clause in clauses])
    assert got == expected


def test_wnn2():
    clauses = akaza.convert("わたし")
    hiragana_len = len([True for node in clauses[0] if node.word == 'わたし'])
    for node in clauses[0]:
        print(node)
    assert hiragana_len == 1
