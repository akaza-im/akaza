import marisa_trie

trie = marisa_trie.BytesTrie()
trie.load('akaza_data/data/system_dict.trie')

for yomi, kanji_bytes in trie.items():
    kanjis = kanji_bytes.decode('utf-8')
    print(f"{yomi} {kanjis}")
