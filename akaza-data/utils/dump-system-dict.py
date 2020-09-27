import datetime
import pathlib
import subprocess

import marisa_trie

hash = subprocess.run(["git", "rev-parse", "--short", 'HEAD'], capture_output=True).stdout.decode('utf-8')
sig = datetime.datetime.now().strftime('%Y%m%d-%H%M') + "-" + hash

trie = marisa_trie.BytesTrie()
trie.load('akaza_data/data/system_dict.trie')

pathlib.Path('work/dump').mkdir(exist_ok=True, parents=True)
with open(f"work/dump/{sig}-dict.txt", 'w') as wfp:
    for yomi, kanji_bytes in trie.items():
        kanjis = kanji_bytes.decode('utf-8')
        wfp.write(f"{yomi} /{kanjis}/\n")
