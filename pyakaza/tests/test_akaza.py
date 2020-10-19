import logging
from tempfile import TemporaryDirectory
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from pyakaza.bind import Akaza, GraphResolver, BinaryDict, SystemUnigramLM, SystemBigramLM, Node, UserLanguageModel, \
    Slice, RomkanConverter


logging.basicConfig(level=logging.DEBUG)


def test_wnn():
    tmpdir = TemporaryDirectory()

    user_language_model = UserLanguageModel(
        tmpdir.name + "/uni",
        tmpdir.name + "/bi"
    )

    system_unigram_lm = SystemUnigramLM()
    system_unigram_lm.load("../akaza-data/data/lm_v2_1gram.trie")

    system_bigram_lm = SystemBigramLM()
    system_bigram_lm.load("../akaza-data/data/lm_v2_2gram.trie")

    system_dict = BinaryDict()
    system_dict.load("../akaza-data/data/system_dict.trie")

    single_term = BinaryDict()
    single_term.load("../akaza-data/data/single_term.trie")

    resolver = GraphResolver(
        user_language_model,
        system_unigram_lm,
        system_bigram_lm,
        [system_dict],
        [single_term],
    )
    romkanConverter = RomkanConverter({})
    akaza = Akaza(resolver, romkanConverter)

    src = u'わたしのなまえはなかのです。'
    expected = '私の名前は中野です。'

    print(akaza.get_version())

    got = akaza.convert(src, None)

    assert ''.join([c[0].get_word() for c in got]) == expected
