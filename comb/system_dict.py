import logging
import os
import pathlib
import time

import marisa_trie
from gi.repository import GLib

from comb import combromkan
from comb.skkdict import parse_skkdict, merge_skkdict
from comb.config import DICTIONARY_DIR

BOIN = set(['a', 'i', 'u', 'e', 'o'])


def get_mtime(fname):
    try:
        return os.path.getmtime(fname)
    except FileNotFoundError:
        return -1


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
            (DICTIONARY_DIR + '/SKK-JISYO.katakana', 'utf-8'),
        ]

        cache_file = self.cache_file()
        cache_file_mtime = get_mtime(cache_file)
        dict_max_mtime = max([get_mtime(x[0]) for x in dicts])
        self.logger.info(f"Cache file: {cache_file_mtime}, {dict_max_mtime}")
        if cache_file_mtime >= dict_max_mtime:
            self.logger.info("loading cache dictionary")
            trie = marisa_trie.BytesTrie()
            trie.load(cache_file)
            self.trie = trie
            self.logger.info("loaded cache dictionary")
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
