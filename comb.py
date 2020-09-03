import jaconv
import os
import re
import time

import combromkan


def parse_skkdict(path, encoding='euc-jp'):
    result = {}

    with open(path, 'r', encoding=encoding) as fp:
        for line in fp:
            if line.startswith(';;'):
                continue

            m = line.strip().split(' ', 1)
            if len(m) == 2:
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

    def get_candidates(self, src, hiragana):
        candidates = []

        for keyword in [src, hiragana]:
            if keyword in self.dict:
                got = self.dict[keyword]
                self.logger.debug("GOT: %s" % str(got))
                for e in got:
                    candidates.append([e, e])

        return candidates

    def add_entry(self, roma, kanji):
        kana = combromkan.to_hiragana(roma)

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

        # 非同期でかくようにしたほうが better.
        self.save()
        self.logger.info("SAVED! %s" % str(self.dict))

    def save(self):
        pass


class Comb:
    def __init__(self, logger, user_dict):
        self.logger = logger
        self.dictionaries = []

        # TODO: load configuration file.
        self.load_dict('/home/tokuhirom/dotfiles/skk/SKK-JISYO.tokuhirom', encoding='utf-8')
        self.load_dict('/usr/share/skk/SKK-JISYO.L')
        self.load_dict('/usr/share/skk/SKK-JISYO.jinmei')
        self.load_dict('/home/tokuhirom/dotfiles/skk/SKK-JISYO.jawiki', encoding='utf-8')

        self.user_dict = user_dict

    def load_dict(self, fname, encoding='euc-jp'):
        try:
            self.logger.info("loading dictionary: %s" % fname)
            t0 = time.time()
            got = parse_skkdict(fname, encoding)
            self.dictionaries.append(got)
            self.logger.info("LOADed JISYO: %d in %f sec" % (len(got), time.time() - t0))
        except:
            self.logger.error("cannot LOAD JISYO %s" % fname, exc_info=True)

    def convert(self, src):
        hiragana = combromkan.to_hiragana(src)
        katakana = jaconv.hira2kata(hiragana)

        # TODO load user dictionary

        candidates = self.user_dict.get_candidates(src, hiragana) + self.get_candidates(src, hiragana)

        candidates.insert(0, [hiragana, hiragana])
        candidates.insert(2, [katakana, katakana])
        if src[0].isupper():
            self.logger.info(f"HAHAH! starting charactger is upper!いめ")
            candidates.insert(0, [src, src])
        else:
            self.logger.info(f"HAHAH! starting charactger is not upper!いめ {src[0]}")
            candidates.append([src, src])

        return candidates

    # src は /better/ みたいな英単語を検索するためにワタシテイルです。
    def get_candidates(self, src, hiragana):
        candidates = []

        for keyword in [src, hiragana]:
            for dictionary in self.dictionaries:
                if keyword in dictionary:
                    got = dictionary[keyword]
                    self.logger.debug("GOT: %s" % str(got))
                    for e in got:
                        candidates.append([e, e])

        return candidates
