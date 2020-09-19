import logging
import re
import time
from logging import Logger
from typing import List

import jaconv
from akaza.graph import GraphResolver
from akaza.node import Node

from akaza.romkan import RomkanConverter

# 子音だが、N は NN だと「ん」になるので処理しない。
TRAILING_CONSONANT_PATTERN = re.compile(r'^(.*?)([qwrtypsdfghjklzxcvbm]+)$')


class Akaza:
    resolver: GraphResolver
    logger: Logger

    def __init__(self,
                 resolver: GraphResolver,
                 romkan: RomkanConverter,
                 logger: Logger = logging.getLogger(__name__)):
        self.logger = logger
        self.resolver = resolver
        self.romkan = romkan

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

        hiragana: str = self.romkan.to_hiragana(src)

        # 末尾の子音を変換対象外とする。
        m = TRAILING_CONSONANT_PATTERN.match(hiragana)
        if m:
            hiragana = m[1]
            consonant = m[2]
            print(f"{hiragana} {consonant}")

        katakana: str = jaconv.hira2kata(hiragana)
        self.logger.info(f"convert: src={src} hiragana={hiragana} katakana={katakana}")

        t0 = time.time()
        ht = dict(self.resolver.lookup(hiragana))
        graph = self.resolver.graph_construct(hiragana, ht, force_selected_clause)
        self.logger.info(
            f"graph_constructed: src={src} hiragana={hiragana} katakana={katakana}: {time.time() - t0} seconds")
        clauses = self.resolver.viterbi(graph)
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
