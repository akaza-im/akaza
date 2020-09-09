import re
import jaconv
import sys

sys.path.append('../')

from comb.skkdict import write_skkdict

# https://www.ncbi.nlm.nih.gov/staff/beck/charents/unicode/30A0-30FF.html
# 30FB  ・ は除外。30FC ー は先頭から除外したい。
KATAKANA_BLOCK = r'\u30A1-\u30FA\u30FD-\u30FF'
KATAKANA_PATTERN = re.compile(r'^[' + KATAKANA_BLOCK + '][' + KATAKANA_BLOCK + '\u30FC]*$')

HIRAGANA_BLOCK = r'\u3041-\u309F'
HIRAGANA_PATTERN = re.compile(r'^[' + HIRAGANA_BLOCK + ']+$')


def scan_kana():
    with open('jawiki.vocab', 'r') as rfp:
        for line in rfp:
            word = line.rstrip()
            m = word.split('/')
            if len(m) != 2:
                continue
            # IGNORE 日本/にっぽん
            word, kana = m
            if KATAKANA_PATTERN.match(word):
                # ワード/わーど
                yield word, kana
            elif HIRAGANA_PATTERN.match(word):
                # やよい/やよい
                yield word, kana


okuri_nasi = {}

for word, kana in scan_kana():
    if kana == 'UNK':
        continue

    if kana not in okuri_nasi:
        okuri_nasi[kana] = []
    okuri_nasi[kana].append(word)

write_skkdict('SKK-JISYO.kana', {}, okuri_nasi)

