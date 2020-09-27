import marisa_trie

from akaza_data_utils import get_sig, mkdir_p


def dump_system_dict(sig):
    trie = marisa_trie.BytesTrie()
    trie.load('akaza_data/data/system_dict.trie')

    mkdir_p('work/dump')
    with open(f"work/dump/{sig}-dict.txt", 'w') as wfp:
        for yomi, kanji_bytes in trie.items():
            kanjis = kanji_bytes.decode('utf-8')
            wfp.write(f"{yomi} /{kanjis}/\n")


def dump_system_lm(type_name: str, sig):
    print(f"Type: {type_name}")

    trie = marisa_trie.RecordTrie('<f')
    trie.mmap(f'akaza_data/data/system_language_model.{type_name}.trie')
    print(f"Len: {type_name} {len(trie)}")

    mkdir_p('work/dump')
    with open(f"work/dump/{sig}-{type_name}.txt", 'w') as wfp:
        for key, score in trie.items():
            wfp.write(f"{key} {score}\n")


def main():
    sig = get_sig()

    dump_system_dict(sig)

    dump_system_lm('1gram', sig)
    dump_system_lm('2gram', sig)
    dump_system_lm('3gram', sig)


if __name__ == '__main__':
    main()
