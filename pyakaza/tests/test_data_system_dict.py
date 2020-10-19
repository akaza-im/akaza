import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from pyakaza.bind import BinaryDict

system_dict = BinaryDict()
system_dict.load("../akaza-data/data/system_dict.trie")


def test_system_dict():
    s = system_dict.find_kanjis('れいわ')
    assert '令和' in s

    beer = system_dict.find_kanjis('びーる')
    assert '🍻' not in beer


def test_system_dict2():
    assert system_dict.find_kanjis('あいう') == ['藍宇']
    assert len(system_dict.find_kanjis('あい')) > 7
