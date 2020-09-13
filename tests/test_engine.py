from tempfile import NamedTemporaryFile

import pytest

from comb.engine import Comb
from comb.system_dict import SystemDict
from comb.user_language_model import UserLanguageModel

tmpfile = NamedTemporaryFile(delete=False)
user_language_model = UserLanguageModel(tmpfile.name)
system_dict = SystemDict.create()

comb = Comb(user_language_model=user_language_model, system_dict=system_dict, user_dict=None)


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
    clauses = comb.convert(src)
    got = ''.join([clause[0].word for clause in clauses])
    assert got == expected


def test_wnn2():
    clauses = comb.convert("わたし")
    hiragana_len = len([True for node in clauses[0] if node.word == 'わたし'])
    for node in clauses[0]:
        print(node)
    assert hiragana_len == 1
