import logging
import os

from comb import combromkan
from comb.skkdict import parse_skkdict, write_skkdict


class UserDict:
    def __init__(self, path, logger=logging.getLogger(__name__)):
        self.path = path
        self.logger = logger
        if os.path.isfile(path):
            self.dict_ari, self.dict_nasi = parse_skkdict(path, encoding='utf-8')
        else:
            self.dict_ari, self.dict_nasi = {}, {}

    def get_candidates(self, src, hiragana):
        candidates = []

        for keyword in [src, hiragana]:
            if keyword in self.dict_nasi:
                got = self.dict_nasi[keyword]
                self.logger.debug("GOT: %s" % str(got))
                for e in got:
                    candidates.append([e, e])

        return candidates

    def add_entry(self, roma, kanji):
        self.logger.info(f"add user_dict entry: roma='{roma}' kanji='{kanji}'")
        kana = combromkan.to_hiragana(roma)

        if kana in self.dict_nasi:
            e = self.dict_nasi[kana]
            if kanji in e:
                # イチバンマエにもっていく。
                e.remove(kanji)
                e.insert(0, kanji)
            else:
                self.dict_nasi[kana].insert(0, kanji)
        else:
            self.dict_nasi[kana] = [kanji]

        # 非同期でかくようにしたほうが better.
        self.save()
        self.logger.info("SAVED!")

    def save(self):
        write_skkdict(self.path, self.dict_ari, self.dict_nasi)
