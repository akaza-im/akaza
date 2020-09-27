import pathlib

import marisa_trie


def process(type_name: str):
    print(f"Type: {type_name}")

    trie = marisa_trie.RecordTrie('<f')
    trie.mmap(f'akaza_data/data/system_language_model.{type_name}.trie')
    print(f"Len: {type_name} {len(trie)}")

    pathlib.Path('work/dump').mkdir(exist_ok=True, parents=True)
    with open(f"work/dump/{type_name}.txt", 'w') as wfp:
        for key, score in trie.items():
            wfp.write(f"{key} {score}\n")


def main():
    process('1gram')
    process('2gram')
    process('3gram')


if __name__ == '__main__':
    main()
