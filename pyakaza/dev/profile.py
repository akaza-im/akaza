#!/usr/bin/env python

import pathlib
from tempfile import TemporaryDirectory

import sys

path = str(pathlib.Path(__file__).parent.parent.absolute())
print(path)
sys.path.insert(0, path)
sys.path.insert(0, pathlib.Path(__file__).parent.parent.parent.joinpath('akaza-data').absolute())

from pyakaza.bind import Akaza, GraphResolver, BinaryDict, SystemUnigramLM, SystemBigramLM, Node, UserLanguageModel, \
    RomkanConverter

tmpdir = TemporaryDirectory()

user_language_model = UserLanguageModel(
    tmpdir.name + "/uni",
    tmpdir.name + "/bi"
)

system_unigram_lm = SystemUnigramLM()
system_unigram_lm.load("../akaza-data/akaza_data/data/lm_v2_1gram.trie")

system_bigram_lm = SystemBigramLM()
system_bigram_lm.load("../akaza-data/akaza_data/data/lm_v2_2gram.trie")

system_dict = BinaryDict()
system_dict.load("../akaza-data/akaza_data/data/system_dict.trie")

single_term = BinaryDict()
single_term.load("../akaza-data/akaza_data/data/single_term.trie")

resolver = GraphResolver(
    user_language_model,
    system_unigram_lm,
    system_bigram_lm,
    [system_dict],
    [single_term],
)
romkanConverter = RomkanConverter({})
akaza = Akaza(resolver, romkanConverter)

# for i in range(10):
#     for line in ['watasinonamaehanakanodseu', 'tonarinokyakuhayokukakikuukyakuda', 'kyounotenkihakumoridana.',
#                  'souieba,asitanotenkihadonoyounakanjininarunodarouka,watasinihamattaakuwakaranai.',
#                  # '長くなってくると、変換に如実に時間がかかるようになってくる。'
#                  'nagakunattekuruto,henkannninyojitunijikanngakakaruyouninattekuru.'
#                  ]:
#         akaza.convert(line)

from timethese import cmpthese, pprint_cmp

print("START")

cmp_res_dict = cmpthese(
    10,
    {
        "term1": lambda: akaza.convert('nagakunattekuruto,henkannninyojitunijikanngakakaruyouninattekuru.', None),
    },
    repeat=10,
)
print(pprint_cmp(cmp_res_dict))
