import logging
from tempfile import TemporaryDirectory
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent.joinpath('../').absolute().resolve()))

import pytest
from pyakaza.bind import Akaza, GraphResolver, BinaryDict, SystemUnigramLM, SystemBigramLM, Node, UserLanguageModel

system_unigram_lm = SystemUnigramLM()
system_unigram_lm.load("../akaza-data/akaza_data/data/lm_v2_1gram.trie")

system_bigram_lm = SystemBigramLM()
system_bigram_lm.load("../akaza-data/akaza_data/data/lm_v2_2gram.trie")

tmpdir = TemporaryDirectory()
user_language_model = UserLanguageModel(
    tmpdir.name + "/uni",
    tmpdir.name + "/bi"
)

system_dict = BinaryDict()
system_dict.load("../akaza-data/akaza_data/data/system_dict.trie")

single_term = BinaryDict()
single_term.load("../akaza-data/akaza_data/data/single_term.trie")

src = 'わたしのなまえはなかのです'
expected = '私の名前は中野です'

resolver = GraphResolver(
    user_language_model,
    system_unigram_lm,
    system_bigram_lm,
    [system_dict],
    [single_term],
)
graph = resolver.graph_construct(src, None)

resolver.fill_cost(graph)
graph.dump()
clauses = resolver.find_nbest(graph)
