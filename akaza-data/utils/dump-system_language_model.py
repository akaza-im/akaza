import pathlib
import subprocess
import datetime

import marisa_trie


def process(type_name: str, sig):
    print(f"Type: {type_name}")

    trie = marisa_trie.RecordTrie('<f')
    trie.mmap(f'akaza_data/data/system_language_model.{type_name}.trie')
    print(f"Len: {type_name} {len(trie)}")

    pathlib.Path('work/dump').mkdir(exist_ok=True, parents=True)
    with open(f"work/dump/{sig}-{type_name}.txt", 'w') as wfp:
        for key, score in trie.items():
            wfp.write(f"{key} {score}\n")


def main():
    hash = subprocess.run(["git", "rev-parse", "--short", 'HEAD'], capture_output=True).stdout.decode('utf-8')
    sig = datetime.datetime.now().strftime('%Y%m%d-%H%M') + "-" + hash

    process('1gram', sig)
    process('2gram', sig)
    process('3gram', sig)


if __name__ == '__main__':
    main()
