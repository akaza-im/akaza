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

            result[yomi] = set(kanjis)

    return result

class Comb:
    def __init__(self, logger):
        self.logger = logger
        try:
            self.l_jisyo = parse_skkdict('/usr/share/skk/SKK-JISYO.L')
            self.logger.info("LOADed JISYO")
        except:
            self.logger.debug("cannot LOAD JISYO %s" % sys.exc_info()[0])
        if not self.l_jisyo:
            self.l_jisyo = {}

    def convert(self, src):
        hiragana = romkan.to_hiragana(src).replace('.', '。').replace(',', '、')
        katakana = romkan.to_kana(src).replace('.', '。').replace(',', '、')

        retval = [
            # KANA / KANJI KOUHO
            (hiragana, hiragana),
            (katakana, katakana),
        ]
        if hiragana in self.l_jisyo:
            got = self.l_jisyo[hiragana]
            self.logger.debug("GOT: %s" % str(got))
            for e in got:
                retval.append([e, e])

        retval.append([src, src])
        return retval

