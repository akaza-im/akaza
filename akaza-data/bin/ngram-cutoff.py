import json


def main(bigram_cutoff=3):
    with open('work/jawiki.2gram-merged.json') as rfp, \
            open('work/jawiki.2gram.json', 'w') as wfp:
        dat = json.load(rfp)
        print(f"Loaded {rfp}")
        remove_entries = []
        for word1 in dat.keys():
            for word2, cnt in dat[word1].items():
                if cnt < bigram_cutoff:
                    remove_entries.append((word1, word2))
        for word1, word2 in remove_entries:
            # print(f"Removing {word1}---{word2}")
            # result[word1].pop(word2, None)
            del dat[word1][word2]
            assert word2 not in dat[word1]

        json.dump(dat, wfp, ensure_ascii=False, indent=1, sort_keys=True)


if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(prog='PROG')
    parser.add_argument('--bigram-cutoff', nargs=1, default=[3], type=int)
    args = parser.parse_args()

    main(bigram_cutoff=args.bigram_cutoff[0])
