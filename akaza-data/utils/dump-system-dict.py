import pathlib

import marisa_trie

trie = marisa_trie.BytesTrie()
trie.load('akaza_data/data/system_dict.trie')

pathlib.Path('work/dump').mkdir(exist_ok=True, parents=True)
with open(f"work/dump/dict.txt", 'w') as wfp:
    for yomi, kanji_bytes in trie.items():
        kanjis = kanji_bytes.decode('utf-8')
        wfp.write(f"{yomi} {kanjis}\n")
