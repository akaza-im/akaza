import jaconv
import os
import re
import time
import logging

import marisa_trie

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


def write_skkdict(outfname, dictionary, encoding='utf-8'):
    with open(outfname, 'w', encoding=encoding) as ofh:
        for yomi in sorted(dictionary.keys()):

            kanjis = dictionary[yomi]
            if len(kanjis) != 0:
                ofh.write("%s /%s/\n" % (yomi, '/'.join(kanjis)))


def merge_skkdict(dicts):
    result = {}

    for dic in dicts:
        for k, v in dic.items():
            if k not in result:
                result[k] = []
            result[k].extend(v)

    return result


class SystemDict:
    def __init__(self, logger=logging.getLogger(__name__)):
        self.logger = logger
        self._load()

    def _load(self):
        # TODO: load configuration file.
        dicts = [
            ('/home/tokuhirom/dotfiles/skk/SKK-JISYO.tokuhirom', 'utf-8'),
            ('/usr/share/skk/SKK-JISYO.L', 'euc-jp'),
            ('/usr/share/skk/SKK-JISYO.jinmei', 'euc-jp'),
            ('/home/tokuhirom/dotfiles/skk/SKK-JISYO.jawiki', 'utf-8'),
        ]
        try:
            self.logger.info("loading dictionaries")
            t0 = time.time()
            t = []
            dictionary = merge_skkdict([parse_skkdict(fname, encoding) for fname, encoding in dicts])
            for k, v in dictionary.items():
                t.append((k, '/'.join(v).encode('utf-8')))
            self.trie = marisa_trie.BytesTrie(t)
            self.logger.info(f"LOADed JISYO: in {time.time() - t0:f} sec")
        except:
            self.logger.error("cannot LOAD JISYO", exc_info=True)

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
        self.logger.info(f"add user_dict entry: roma='{roma}' kanji='{kanji}'")
        kana = combromkan.to_hiragana(roma)

        if kana in self.dict:
            e = self.dict[kana]
            if kanji in e:
                # イチバンマエにもっていく。
                e.remove(kanji)
                e.insert(0, kanji)
            else:
                self.dict[kana].insert(0, kanji)
        else:
            self.dict[kana] = [kanji]

        # 非同期でかくようにしたほうが better.
        self.save()
        self.logger.info("SAVED! %s" % str(self.dict))

    def save(self):
        write_skkdict(self.path, self.dict)


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
            candidates.insert(2, [katakana, katakana])

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
    print(comb.convert('henkandekiru'))
    # print(comb.convert('hituyoudayo'))
    print(list(d.get_candidates('henkandekiru', 'へんかんできる')))
    # print(list(d.get_candidates('hiragana', 'ひらがな')))
    # print(list(d.get_candidates('buffer', 'ぶっふぇr')))
