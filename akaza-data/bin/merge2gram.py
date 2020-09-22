import sys
import glob
import json
import time


def main():
    t0 = time.time()

    files = glob.glob('work/2gram/*/wiki_*.2gram.json')
    finished = 0
    result = {}
    for file in files:
        with open(file) as fp:
            print(f"Reading {file} {finished}/{len(files)}")
            dat = json.load(fp)
            for word1, word2items in dat.items():
                if word1 not in result:
                    result[word1] = {}
                for word2, cnt in word2items.items():
                    if word2 not in result[word1]:
                        result[word1][word2] = 0
                    result[word1][word2] += cnt
        finished += 1

    print("Writing result")
    with open('work/jawiki.2gram.json', 'w') as wfp:
        json.dump(result, wfp, ensure_ascii=False, indent=1, sort_keys=True)

    print(f"Elapsed: {time.time() - t0} seconds")


if __name__ == '__main__':
    main()
