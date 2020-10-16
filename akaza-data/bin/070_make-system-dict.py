import re
import sys

sys.path.append('../')

from skkdictutils import parse_skkdict, merge_skkdict, ari2nasi
from akaza_data_utils import copy_snapshot

# jawiki.vocab から system_dict.trie を作成する。


# https://www.ncbi.nlm.nih.gov/staff/beck/charents/unicode/30A0-30FF.html
# 30FB  ・ は除外。30FC ー は先頭から除外したい。
KATAKANA_BLOCK = r'\u30A1-\u30FA\u30FD-\u30FF'
KATAKANA_PATTERN = re.compile(r'^[' + KATAKANA_BLOCK + '][' + KATAKANA_BLOCK + '\u30FC]*$')

HIRAGANA_BLOCK = r'\u3041-\u309F'
HIRAGANA_PATTERN = re.compile(r'^[' + HIRAGANA_BLOCK + ']+$')


def scan_vocab():
    with open('work/jawiki.vocab', 'r') as rfp:
        for line in rfp:
            word = line.rstrip()
            m = word.split('/')
            if len(m) != 2:
                continue

            word, kana = m
            if kana == 'UNK':
                continue
            yield word, kana


def make_vocab_dict():
    okuri_nasi = {}

    for word, kana in scan_vocab():
        if kana not in okuri_nasi:
            okuri_nasi[kana] = []
        okuri_nasi[kana].append(word)

    return okuri_nasi


def make_lisp_dict():
    return {
        'きょう': [
            '(strftime (current-datetime) "%Y-%m-%d")',
            '(strftime (current-datetime) "%Y年%m月%d日")',
            '(strftime (current-datetime) "%Y年%m月%d日(%a)")',
        ]
    }


def write_dict(ofname, dicts):
    merged_dict = merge_skkdict(dicts)
    with open(ofname, 'w') as wfp:
        for yomi, kanjis in merged_dict.items():
            kanjis = '/'.join(kanjis)
            wfp.write(f"{yomi} {kanjis}\n")
    copy_snapshot(ofname)


def make_system_dict():
    dictionary_sources = [
        # 先の方が優先される
        ('skk-dev-dict/SKK-JISYO.L', 'euc-jp'),
        ('skk-dev-dict/SKK-JISYO.jinmei', 'euc-jp'),
        ('skk-dev-dict/SKK-JISYO.station', 'euc-jp'),
        ('jawiki-kana-kanji-dict/SKK-JISYO.jawiki', 'utf-8'),
        ('dict/SKK-JISYO.akaza', 'utf-8'),
    ]
    dicts = []

    for path, encoding in dictionary_sources:
        ari, nasi = parse_skkdict(path, encoding)
        dicts.append(nasi)
        dicts.append(ari2nasi(ari))

    dicts.append(make_vocab_dict())

    write_dict('work/jawiki.system_dict.txt', dicts)


def make_single_term_dict():
    dictionary_sources = [
        # 先の方が優先される
        ('skk-dev-dict/SKK-JISYO.emoji', 'utf-8'),
        ('skk-dev-dict/zipcode/SKK-JISYO.zipcode', 'euc-jp'),
    ]
    dicts = []

    for path, encoding in dictionary_sources:
        ari, nasi = parse_skkdict(path, encoding)
        dicts.append(nasi)
        dicts.append(ari2nasi(ari))
    dicts.append(make_lisp_dict())
    write_dict("work/jawiki.single_term.txt", dicts)


def main():
    make_single_term_dict()
    make_system_dict()


if __name__ == '__main__':
    main()
