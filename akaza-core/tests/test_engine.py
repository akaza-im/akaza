import pathlib
import sys
from tempfile import NamedTemporaryFile

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

import pytest
from akaza import Akaza
from akaza.user_language_model import UserLanguageModel
from akaza.graph_resolver import GraphResolver
from akaza.romkan import RomkanConverter
from akaza_data.systemlm_loader import BinaryDict, SystemLM

tmpfile = NamedTemporaryFile(delete=False)
user_language_model = UserLanguageModel(tmpfile.name)

system_dict = BinaryDict()
system_dict.load("../akaza-data/akaza_data/data/system_dict.trie")

system_language_model = SystemLM()
system_language_model.load(
    "../akaza-data/akaza_data/data/lm_v2_1gram.trie",
    "../akaza-data/akaza_data/data/lm_v2_2gram.trie"
)

emoji_dict = BinaryDict()
emoji_dict.load("../akaza-data/akaza_data/data/single_term.trie")

resolver = GraphResolver(
    system_language_model=system_language_model,
    user_language_model=user_language_model,
    normal_dicts=[system_dict],
    single_term_dicts=[emoji_dict],
)

romkan = RomkanConverter()

akaza = Akaza(resolver=resolver, romkan=romkan)


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
    print(user_language_model.get_unigram_cost('子音/しいん'))
    assert got == expected


def test_wnn2():
    clauses = akaza.convert("わたし")
    hiragana_len = len([True for node in clauses[0] if node.word == 'わたし'])
    for node in clauses[0]:
        print(node)
    assert hiragana_len == 1
