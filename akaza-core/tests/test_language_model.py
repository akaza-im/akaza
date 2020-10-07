import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../../akaza-data/').absolute().resolve()))

from akaza.node import Node
from akaza.user_language_model import UserLanguageModel

from tempfile import TemporaryDirectory

from akaza_data.systemlm_loader import BinaryDict, SystemUnigramLM, SystemBigramLM

system_unigram_lm = SystemUnigramLM()
system_unigram_lm.load("../akaza-data/akaza_data/data/lm_v2_1gram.trie")

system_bigram_lm = SystemBigramLM()
system_bigram_lm.load("../akaza-data/akaza_data/data/lm_v2_2gram.trie")


def test_read():
    tmpdir = TemporaryDirectory()
    user_language_model = UserLanguageModel(tmpdir.name + "/foobar.dict")
    user_language_model.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    user_language_model.add_entry([Node(start_pos=0, word='単語', yomi='たんご')])
    user_language_model.add_entry([Node(start_pos=0, word='熟語', yomi='じゅくご')])

    assert Node(start_pos=0, word='単語', yomi='たんご'
                ).calc_node_cost(
        user_language_model, system_unigram_lm
    ) > Node(
        start_pos=0, word='熟語', yomi='じゅくご'
    ).calc_node_cost(user_language_model, system_unigram_lm)
