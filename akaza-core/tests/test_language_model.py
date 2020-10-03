import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from akaza.node import Node
from akaza.user_language_model import UserLanguageModel

from tempfile import TemporaryDirectory

from akaza.language_model import LanguageModel
from akaza_data import SystemLanguageModel


def test_read():
    tmpdir = TemporaryDirectory()
    d = UserLanguageModel(tmpdir.name + "/foobar.dict")
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    d.add_entry([Node(start_pos=0, word='熟語', yomi='じゅくご')])

    d = LanguageModel(SystemLanguageModel.load(), d)
    assert d.calc_node_cost(Node(start_pos=0, word='単語', yomi='たんご')) > d.calc_node_cost(
        Node(start_pos=0, word='熟語', yomi='じゅくご'))
