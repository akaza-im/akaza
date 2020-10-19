import logging
from tempfile import TemporaryDirectory
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from pyakaza.bind import Akaza, GraphResolver, BinaryDict, SystemUnigramLM, SystemBigramLM, Node, UserLanguageModel, \
    Slice, build_romkan_converter


logging.basicConfig(level=logging.DEBUG)

emoji_dict = BinaryDict()
emoji_dict.load("../akaza-data/data/single_term.trie")


def test_system_dict():
    s = emoji_dict.find_kanjis('すし')
    assert '🍣' in s
