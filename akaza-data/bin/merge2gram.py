import glob
import json
import time


def main(bigram_cutoff=3):
    print(f"bigram-cutoff={bigram_cutoff}")

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

    # assertion
    #  basic entries.
    assert 'で/で' in result['中野/なかの']

    with open('work/jawiki.2gram-merged.json', 'w') as wfp:
        json.dump(result, wfp, ensure_ascii=False, indent=1, sort_keys=True)

    print(f"Elapsed: {time.time() - t0} seconds")


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(prog='PROG')
    parser.add_argument('--bigram-cutoff', nargs=1, default=[3], type=int)
    args = parser.parse_args()

    main(bigram_cutoff=args.bigram_cutoff[0])
