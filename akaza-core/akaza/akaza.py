import logging
import re
import time
from logging import Logger
from typing import List, Any, Optional

import jaconv

from akaza import romkan
from akaza.graph import graph_construct, viterbi, lookup
from akaza.language_model import LanguageModel
from akaza.node import Node
from akaza_data.system_dict import SystemDict
from akaza_data.system_language_model import SystemLanguageModel
from akaza.user_dict import UserDict
from akaza.user_language_model import UserLanguageModel

# 子音だが、N は NN だと「ん」になるので処理しない。
TRAILING_CONSONANT_PATTERN = re.compile(r'^(.*?)([qwrtypsdfghjklzxcvbm]+)$')


class Akaza:
    user_dict: Optional[UserDict]
    logger: Logger
    dictionaries: List[Any]

    def __init__(self,
                 system_language_model: SystemLanguageModel,
                 system_dict: SystemDict,
                 user_language_model: UserLanguageModel,
                 user_dict: Optional[UserDict] = None,
                 logger: Logger = logging.getLogger(__name__)):
        assert user_language_model
        self.logger = logger
        self.dictionaries = []
        self.user_language_model = user_language_model
        self.system_dict = system_dict
        self.user_dict = user_dict

        self.language_model = LanguageModel(system_language_model, user_language_model)

    # 連文節変換するバージョン。
    def convert(self, src: str, force_selected_clause: List[slice] = None) -> List[List[Node]]:
        self.logger.info(f"convert: {force_selected_clause}")

        if len(src) > 0 and src[0].isupper() and not force_selected_clause:
            # 最初の文字が大文字で、文節の強制指定がない場合、アルファベット強制入力とする。
            return [[
                Node(
                    start_pos=0,
                    word=src,
                    yomi=src,
                )
            ]]

        hiragana: str = romkan.to_hiragana(src)

        # 末尾の子音を変換対象外とする。
        m = TRAILING_CONSONANT_PATTERN.match(hiragana)
        if m:
            hiragana = m[1]
            consonant = m[2]
            print(f"{hiragana} {consonant}")

        katakana: str = jaconv.hira2kata(hiragana)
        self.logger.info(f"convert: src={src} hiragana={hiragana} katakana={katakana}")

        t0 = time.time()
        ht = dict(lookup(hiragana, self.system_dict, self.user_language_model, self.user_dict))
        graph = graph_construct(hiragana, ht, force_selected_clause)
        self.logger.info(
            f"graph_constructed: src={src} hiragana={hiragana} katakana={katakana}: {time.time() - t0} seconds")
        clauses = viterbi(graph, self.language_model)
        self.logger.info(
            f"converted: src={src} hiragana={hiragana} katakana={katakana}: {time.time() - t0} seconds")

        if m:
            clauses.append([Node(
                start_pos=len(src),
                word=consonant,
                yomi=consonant,
            )])
            return clauses
        else:
            return clauses
