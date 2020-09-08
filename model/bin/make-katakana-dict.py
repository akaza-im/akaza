import re
import jaconv

# https://www.ncbi.nlm.nih.gov/staff/beck/charents/unicode/30A0-30FF.html
# 30FB  ・ は除外。30FC ー は先頭から除外したい。
KATAKANA_BLOCK = r'\u30A1-\u30FA\u30FD-\u30FF'
KATAKANA_PATTERN = re.compile(r'^[' + KATAKANA_BLOCK + '][' + KATAKANA_BLOCK + '\u30FC]*$')


def scan_katakana():
    with open('jawiki.vocab', 'r') as rfp:
        for line in rfp:
            word = line.rstrip()
            m = word.split('/')
            if len(m) > 1:
                word = m[1]
            if KATAKANA_PATTERN.match(word):
                yield word


with open('SKK-JISYO.katakana', 'w') as wfp:
    wfp.write(";; okuri-ari entries.\n")
    wfp.write(";; okuri-nasi entries.\n")
    for word in scan_katakana():
        wfp.write(f"{jaconv.kata2hira(word)} /{word}/\n")
