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
