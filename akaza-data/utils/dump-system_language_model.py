import marisa_trie

trie = marisa_trie.RecordTrie('<f')
trie.mmap('akaza_data/data/system_language_model.trie')

for key, score in trie.items():
    print(f"{key} {score}")
