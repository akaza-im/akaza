from tempfile import NamedTemporaryFile

from comb.combromkan import to_hiragana
import pytest
import marisa_trie
from comb.system_dict import SystemDict
from comb.graph import lookup, graph_construct, viterbi
from comb.engine import Comb
from comb.user_dict import UserDict

tmpfile = NamedTemporaryFile(delete=False)
user_dict = UserDict(tmpfile.name)
system_dict = SystemDict()

comb = Comb(user_dict=user_dict, system_dict=system_dict)


@pytest.mark.parametrize('src, expected', [
    # Wnn で有名なフレーズ。
    ('わたしのなまえはなかのです', '私の名前は中野です'),
    # カタカナ語の処理が出来ていること。
    ('わーど', 'ワード'),
    ('にほん', '日本'),
    ('それなwww', 'それなwww'),
    ('siinn', '子音'),
    ('zh', '←'),
])
def test_wnn(src, expected):
    clauses = comb.convert2(src)
    got = ''.join([clause[0].word for clause in clauses])
    assert got == expected

