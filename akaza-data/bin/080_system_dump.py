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


def main():
    sig = get_sig()

    dump_system_dict(sig)


if __name__ == '__main__':
    main()
