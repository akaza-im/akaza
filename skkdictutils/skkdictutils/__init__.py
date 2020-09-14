import re
from typing import Dict, List

import romkan

__all__ = ['parse_skkdict', 'merge_skkdict', 'ari2nasi', 'write_skkdict', 'expand_okuri']

_BOIN = ['a', 'i', 'u', 'e', 'o']

_LOWER_PATTERN = re.compile('[a-z]')


def parse_skkdict(path: str, encoding: str = 'euc-jp'):
    nasi: Dict[str, List[str]] = {}
    ari: Dict[str, List[str]] = {}
    target = ari

    with open(path, 'r', encoding=encoding) as fp:
        for line in fp:
            if line == ";; okuri-ari entries.\n":
                target = ari
            if line == ";; okuri-nasi entries.\n":
                target = nasi
            if line.startswith(';;'):
                continue

            m = line.strip().split(' ', 1)
            if len(m) == 2:
                yomi: str = m[0]
                kanjis: List[str] = m[1].lstrip('/').rstrip('/').split('/')
                kanjis = [re.sub(';.*', '', k) for k in kanjis]

                target[yomi] = kanjis

    return ari, nasi


def write_skkdict(outfname: str, dict_ari, dict_nasi, encoding: str = 'utf-8'):
    with open(outfname, 'w', encoding=encoding) as ofh:
        ofh.write(";; okuri-ari entries.\n")
        for yomi in sorted(dict_ari.keys()):
            kanjis = dict_ari[yomi]
            if len(kanjis) != 0:
                ofh.write("%s /%s/\n" % (yomi, '/'.join(kanjis)))

        ofh.write(";; okuri-nasi entries.\n")
        for yomi in sorted(dict_nasi.keys()):
            kanjis = dict_nasi[yomi]
            if len(kanjis) != 0:
                ofh.write("%s /%s/\n" % (yomi, '/'.join(kanjis)))


def merge_skkdict(dicts: List[Dict[str, List[str]]]) -> Dict[str, List[str]]:
    result = {}

    for dic in dicts:
        for kana, kanjis in dic.items():
            if kana not in result:
                result[kana] = []
            for kanji in kanjis:
                if kanji not in result[kana]:
                    result[kana].append(kanji)

    return result


def expand_okuri(kana: str, kanjis: List[str]):
    if kana[-1].isalpha():
        if kana[-1] in _BOIN:
            okuri = romkan.to_hiragana(kana[-1])
            yield kana[:-1] + okuri, [kanji + okuri for kanji in kanjis]
        else:
            for b in _BOIN:
                okuri = romkan.to_hiragana(kana[-1] + b)
                if _LOWER_PATTERN.match(okuri):
                    # wu のように、変換できないものは無視する。
                    continue
                yield kana[:-1] + okuri, [kanji + okuri for kanji in kanjis]
    else:
        yield kana, kanjis


def ari2nasi(src):
    retval = {}
    for kana, kanjis in src.items():
        for kkk, vvv in expand_okuri(kana, kanjis):
            retval[kkk] = vvv
    return retval
