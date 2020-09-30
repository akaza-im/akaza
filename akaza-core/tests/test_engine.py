import pathlib
import sys
from tempfile import NamedTemporaryFile

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

import pytest
from akaza.dictionary import Dictionary
from akaza import Akaza
from akaza.user_language_model import UserLanguageModel
from akaza_data.system_dict import SystemDict
from akaza_data.system_language_model import SystemLanguageModel
from akaza.language_model import LanguageModel
from akaza.graph_resolver import GraphResolver
from akaza.romkan import RomkanConverter
from akaza_data.emoji import EmojiDict

tmpfile = NamedTemporaryFile(delete=False)
user_language_model = UserLanguageModel(tmpfile.name)
system_dict = SystemDict.load()

system_language_model = SystemLanguageModel.load()

language_model = LanguageModel(
    system_language_model=system_language_model,
    user_language_model=user_language_model,
)

emoji_dict = EmojiDict.load()

dictionary = Dictionary(
    system_dict=system_dict,
    emoji_dict=emoji_dict,
    user_dicts=[],
)

resolver = GraphResolver(
    language_model=language_model,
    dictionary=dictionary,
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
