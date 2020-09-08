from logging import Logger
from typing import List, Any

import os

import jaconv

from comb import combromkan

from comb.system_dict import SystemDict
from comb.user_dict import UserDict
from comb.graph import graph_construct, viterbi, lookup
from comb.config import MODEL_DIR
import logging
import marisa_trie


class Comb:
    logger: Logger
    dictionaries: List[Any]

    def __init__(self, logger: Logger, user_dict: UserDict, system_dict: SystemDict):
        self.logger = logger
        self.dictionaries = []
        self.user_dict = user_dict
        self.system_dict = system_dict

        self.unigram_score = marisa_trie.RecordTrie('@f')
        self.unigram_score.load(f"{MODEL_DIR}/jawiki.1gram")

        self.bigram_score = marisa_trie.RecordTrie('@f')
        self.bigram_score.load(f"{MODEL_DIR}/jawiki.2gram")

    def convert(self, src):
        hiragana: str = combromkan.to_hiragana(src)
        katakana: str = jaconv.hira2kata(hiragana)

        self.logger.info(f"convert: src={src} hiragana={hiragana} katakana={katakana}")

        candidates = [[hiragana, hiragana]]

        for e in self.user_dict.get_candidates(src, hiragana):
            if e not in candidates:
                candidates.append(e)

        try:
            ht = dict(lookup(hiragana, self.system_dict))
            graph = graph_construct(hiragana, ht, self.unigram_score, self.bigram_score)
            got = viterbi(graph, self.unigram_score)

            phrase = ''.join([x.word for x in got if not x.is_eos()])

            self.logger.info(f"Got phrase: {phrase}")

            if [phrase, phrase] not in candidates:
                candidates.append([phrase, phrase])
        except:
            self.logger.error(f"Cannot convert: {hiragana} {katakana}",
                              exc_info=True)

        if [katakana, katakana] not in candidates:
            candidates.append([katakana, katakana])

        for e in [[x, x] for x in self.system_dict.get_candidates(src, hiragana)]:
            if e not in candidates:
                candidates.append(e)

        if src[0].isupper():
            # 先頭が大文字の場合、それを先頭にもってくる。
            candidates.insert(0, [src, src])
        else:
            # そうじゃなければ、末尾にいれる。
            candidates.append([src, src])

        return candidates


if __name__ == '__main__':
    from gi.repository import GLib
    import pathlib
    import logging

    logging.basicConfig(level=logging.DEBUG)

    configdir = os.path.join(GLib.get_user_config_dir(), 'ibus-comb')
    pathlib.Path(configdir).mkdir(parents=True, exist_ok=True)
    d = SystemDict()
    u = UserDict(os.path.join(configdir, 'user-dict.txt'))
    comb = Comb(logging.getLogger(__name__), u, d)
    # print(comb.convert('henkandekiru'))
    print(comb.convert('watasi'))
    # print(comb.convert('hituyoudayo'))
    # print(list(d.get_candidates('henkandekiru', 'へんかんできる')))
    # print(list(d.get_candidates('warudakumi', 'わるだくみ')))
    # print(list(d.get_candidates('subarasii', 'すばらしい')))
    # print(list(d.get_candidates('watasi', 'わたし')))
    # print(list(d.get_candidates('hiragana', 'ひらがな')))
    # print(list(d.get_candidates('buffer', 'ぶっふぇr')))
