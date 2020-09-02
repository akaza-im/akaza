import romkan
import re
import sys


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


class Comb:
    def __init__(self, logger):
        self.logger = logger
        self.dictionaries = []
        self.load_dict('/usr/share/skk/SKK-JISYO.L')

    def load_dict(self, fname, encoding='euc-jp'):
        try:
            self.logger.info("loading %s" % fname)
            self.dictionaries.append(parse_skkdict(fname, encoding))
            self.logger.info("LOADed JISYO")
        except:
            self.logger.error("cannot LOAD JISYO %s" % (fname), exc_info=True)

    def convert(self, src):
        hiragana = romkan.to_hiragana(src).replace('.', '。').replace(',', '、')
        katakana = romkan.to_kana(src).replace('.', '。').replace(',', '、')

        retval = []

        # TODO load user dictionary

        retval.append([hiragana, hiragana])
        retval.append([katakana, katakana])

        for dictionary in self.dictionaries:
            if hiragana in dictionary:
                got = dictionary[hiragana]
                self.logger.debug("GOT: %s" % str(got))
                for e in got:
                    retval.append([e, e])

        retval.append([src, src])
        return retval

