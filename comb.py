import jaconv
import os
import re
import time
import logging
from gi.repository import GLib
import pathlib

import marisa_trie

import combromkan
from skkdict import parse_skkdict, write_skkdict, merge_skkdict

BOIN = set(['a', 'i', 'u', 'e', 'o'])


class SystemDict:
    def __init__(self, logger=logging.getLogger(__name__)):
        self.logger = logger
        try:
            self._load()
        except:
            self.logger.error("cannot LOAD JISYO", exc_info=True)

    def cache_file(self):
        cachedir = os.path.join(GLib.get_user_cache_dir(), 'ibus-comb')
        pathlib.Path(cachedir).mkdir(parents=True, exist_ok=True)
        return os.path.join(cachedir, 'system-dict.marisa')

    def _load(self):
        # TODO: load configuration file.
        dicts = [
            ('/home/tokuhirom/dotfiles/skk/SKK-JISYO.tokuhirom', 'utf-8'),
            ('/usr/share/skk/SKK-JISYO.L', 'euc-jp'),
            ('/usr/share/skk/SKK-JISYO.jinmei', 'euc-jp'),
            ('/home/tokuhirom/dotfiles/skk/SKK-JISYO.jawiki', 'utf-8'),
        ]

        def get_mtime(fname):
            try:
                return os.path.getmtime(fname)
            except FileNotFoundError:
                return -1

        cache_file = self.cache_file()
        if get_mtime(cache_file) >= max([get_mtime(x[0]) for x in dicts]):
            self.logger.info("loading cache dictionary")
            trie = marisa_trie.BytesTrie()
            trie.load(cache_file)
            self.trie = trie
            return

        self.logger.info("loading dictionaries")
        t0 = time.time()
        t = []
        # TODO cache trie.
        dictionaries = [parse_skkdict(fname, encoding) for fname, encoding in dicts]

        def expand_okuri(kana, kanjis):
            if kana[-1].isalpha():
                if kana[-1] in BOIN:
                    okuri = combromkan.to_hiragana(kana[-1])
                    yield kana[:-1] + okuri, [kanji + okuri for kanji in kanjis]
                else:
                    for b in BOIN:
                        okuri = combromkan.to_hiragana(kana[-1] + b)
                        yield kana[:-1] + okuri, [kanji + okuri for kanji in kanjis]
            else:
                yield kana, kanjis

        def ari2nasi(src):
            retval = {}
            for kana, kanjis in src.items():
                for kkk, vvv in expand_okuri(kana, kanjis):
                    retval[kkk] = vvv
            return retval

        ari_dictionary = merge_skkdict([d[0] for d in dictionaries])
        nasi_dictionary = merge_skkdict(
            [d[1] for d in dictionaries] +
            [ari2nasi(ari_dictionary)]
        )

        for k, v in nasi_dictionary.items():
            t.append((k, '/'.join(v).encode('utf-8')))
        self.trie = marisa_trie.BytesTrie(t)
        self.logger.info(f"LOADed JISYO: in {time.time() - t0:f} sec")
        self.trie.save(cache_file)
        self.logger.info(f"Saved cache file: {cache_file} in {time.time() - t0:f} sec")

    # src は /better/ みたいな英単語を検索するためにワタシテイルです。
    def get_candidates(self, src, hiragana):
        if src in self.trie:
            kanjis = self.trie[src][0].decode('utf-8').split('/')
            for kanji in kanjis:
                yield kanji

        for prefix in reversed(self.trie.prefixes(hiragana)):
            kanjis = self.trie[prefix][0].decode('utf-8').split('/')
            for kanji in kanjis:
                yield kanji + hiragana[len(prefix):]


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


class Comb:
    def __init__(self, logger, user_dict: UserDict, system_dict: SystemDict):
        self.logger = logger
        self.dictionaries = []
        self.user_dict = user_dict
        self.system_dict = system_dict

    def convert(self, src):
        hiragana = combromkan.to_hiragana(src)
        katakana = jaconv.hira2kata(hiragana)

        candidates = [[hiragana, hiragana]]

        for e in self.user_dict.get_candidates(src, hiragana):
            if e not in candidates:
                candidates.append(e)

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
