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


def scan_vocab():
    with open('jawiki.vocab', 'r') as rfp:
        for line in rfp:
            word = line.rstrip()
            print(word)
            m = word.split('/')
            if len(m) != 2:
                continue

            word, kana = m
            if kana == 'UNK':
                continue
            yield word, kana


okuri_nasi = {}

for word, kana in scan_vocab():
    if kana not in okuri_nasi:
        okuri_nasi[kana] = []
    okuri_nasi[kana].append(word)

write_skkdict('SKK-JISYO.jawiki', {}, okuri_nasi)

