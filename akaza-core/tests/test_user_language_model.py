import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from akaza.node import Node
from akaza.user_language_model import UserLanguageModel

from tempfile import TemporaryDirectory


def test_read():
    tmpdir = TemporaryDirectory()
    d = UserLanguageModel(tmpdir.name + "/foobar.dict")
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='熟語', yomi='じゅくご')])
    assert d.unigram == {'単語/たんご': 2, '熟語/じゅくご': 1}
    assert d.total == 3
    assert d.get_unigram_cost('単語/たんご') > d.get_unigram_cost('熟語/じゅくご')


def test_read3():
    tmpdir = TemporaryDirectory()
    user_language_model = UserLanguageModel(tmpdir.name + "/foobar.dict")
    user_language_model.add_entry([
        Node(start_pos=0, word='ヒョイー', yomi='ひょいー'),
    ])

    assert user_language_model.unigram == {'ヒョイー/ひょいー': 1}
    assert user_language_model.has_unigram_cost_by_yomi('ひょいー')


def test_read2():
    tmpdir = TemporaryDirectory()
    d = UserLanguageModel(tmpdir.name + "/foobar.dict")
    d.add_entry([
        Node(start_pos=0, word='私', yomi='わたし'),
        Node(start_pos=1, word='だよ', yomi='だよ'),
    ])
    d.add_entry([
        Node(start_pos=0, word='それは', yomi='それは'),
        Node(start_pos=3, word='私', yomi='わたし'),
        Node(start_pos=4, word='だよ', yomi='だよ'),
    ])
    d.add_entry([
        Node(start_pos=0, word='私', yomi='わたし'),
        Node(start_pos=1, word='です', yomi='です'),
    ])

    assert d.unigram == {'それは/それは': 1, 'だよ/だよ': 2, '私/わたし': 3, 'です/です': 1}
    assert d.total == 7

    assert d.bigram == {'それは/それは\t私/わたし': 1, '私/わたし\tだよ/だよ': 2, '私/わたし\tです/です': 1}
    assert d.bigram_total == {'それは/それは': 1, '私/わたし': 3}
