#!/usr/bin/env python

import pathlib

import sys

path = str(pathlib.Path(__file__).parent.parent.absolute())
print(path)
sys.path.insert(0, path)

import akaza
from akaza.dictionary import Dictionary
from akaza.graph import GraphResolver
from akaza.language_model import LanguageModel
from akaza.romkan import RomkanConverter
from akaza.user_language_model import UserLanguageModel
from akaza_data import SystemLanguageModel, SystemDict

system_language_model = SystemLanguageModel.load()
system_dict = SystemDict.load()
system_dict = SystemDict.load()
system_language_model = SystemLanguageModel.load()

user_language_model_path = pathlib.Path('/tmp/user_language_model')
user_language_model_path.mkdir(parents=True, exist_ok=True, mode=0o700)
user_language_model = UserLanguageModel(str(user_language_model_path))

language_model = LanguageModel(
    system_language_model=system_language_model,
    user_language_model=user_language_model,
)

dictionary = Dictionary(
    system_dict=system_dict,
    user_dicts=[],
)

resolver = GraphResolver(
    dictionary=dictionary,
    language_model=language_model,
)
romkan = RomkanConverter()
akaza = akaza.Akaza(
    resolver=resolver,
    romkan=romkan,
)

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
        "term1": lambda: akaza.convert('nagakunattekuruto,henkannninyojitunijikanngakakaruyouninattekuru.'),
    },
    repeat=10,
)
print(pprint_cmp(cmp_res_dict))
