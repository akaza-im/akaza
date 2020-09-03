import jaconv
import os
import re

import combromkan


def parse_skkdict(path, encoding='euc-jp'):
    result = {}

    with open(path, 'r', encoding=encoding) as fp:
        for line in fp:
            if line.startswith(';;'):
                continue

            m = line.strip().split(' ', 1)
            yomi, kanjis = m
            kanjis = kanjis.lstrip('/').rstrip('/').split('/')
            kanjis = [re.sub(';.*', '', k) for k in kanjis]

            result[yomi] = kanjis

    return result


class UserDict:
    def __init__(self, path, logger):
        self.path = path
        self.logger = logger
        if os.path.isfile(path):
            self.dict = parse_skkdict(path, encoding='utf-8')
        else:
            self.dict = {}

    def has_entry(self, kana):
        return kana in self.dict

    def add_entry(self, kana, kanji):
        if kana in self.dict:
            e = self.dict[kana]
            if kanji in e:
                # イチバンマエにもっていく。
                e.remove(kanji)
                e.insert(0, kanji)
            else:
                self.dict[kana] = kanji
        else:
            self.dict[kana] = [kanji]

    def save(self):
        pass


class Comb:
    def __init__(self, logger):
        self.logger = logger
        self.dictionaries = []

        # TODO: load configuration file.
        self.load_dict('/usr/share/skk/SKK-JISYO.L')
        self.load_dict('/usr/share/skk/SKK-JISYO.jinmei')
        self.load_dict('/home/tokuhirom/dotfiles/skk/SKK-JISYO.jawiki', encoding='utf-8')

    def load_dict(self, fname, encoding='euc-jp'):
        try:
            self.logger.info("loading dictionary: %s" % fname)
            self.dictionaries.append(parse_skkdict(fname, encoding))
            self.logger.info("LOADed JISYO")
        except:
            self.logger.error("cannot LOAD JISYO %s" % (fname), exc_info=True)

    def convert(self, src):
        hiragana = combromkan.to_hiragana(src).replace('.', '。').replace(',', '、')
        katakana = jaconv.hira2kata(hiragana)

        # TODO load user dictionary

        candidates = self.get_candidates(hiragana)

        candidates.insert(0, [hiragana, hiragana])
        candidates.insert(2, [katakana, katakana])
        candidates.append([src, src])

        return candidates

    def get_candidates(self, hiragana):
        candidates = []
        for dictionary in self.dictionaries:
            if hiragana in dictionary:
                got = dictionary[hiragana]
                self.logger.debug("GOT: %s" % str(got))
                for e in got:
                    candidates.append([e, e])
        return candidates
