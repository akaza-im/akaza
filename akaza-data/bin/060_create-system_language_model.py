import glob
import math
import sys
import time

import marisa_trie
# jawiki.1gram.json/jawiki.2gram.json から言語モデルを出力する。
from tqdm import tqdm


def build_model(pattern, cutoff, t0):
    retval = []

    files = glob.glob(pattern)

    # 単語数
    V = 0
    # 総単語出現数
    C = 0

    # additive factor
    alpha = 0.00001

    # stats info.
    print(f"Aggregation phase: {pattern}. elapsed={time.time() - t0}")
    done = 0
    wordcnt = {}
    for file in tqdm(files):
        with open(file) as rfp:
            for line in rfp:
                m = line.split(' ')
                if len(m) == 2:
                    word, cnt = m
                    cnt = int(cnt)
                    C += cnt
                    V += 1
                    wordcnt[word] = wordcnt.get(word, 0) + cnt
        done += 1

    # calc score
    print(f"Scoring phase: {pattern}. elapsed={time.time() - t0}")
    for word, cnt in tqdm(wordcnt.items()):
        if cnt > cutoff:
            score = math.log10((wordcnt[word] + alpha) / (C + alpha * V))
            retval.append((word, (float(score),),))

    default_score = math.log10((0 + alpha) / (C + alpha * V))
    print(f"{pattern} default score is `{default_score}`")

    return retval


def write_trie(path, data):
    trie = marisa_trie.RecordTrie('<f', data)
    print(f"writing {path}.")
    trie.save(path)


def write_model():
    t0 = time.time()

    print(f'[{sys.argv[0]}] # 1gram')
    unigram = build_model('work/ngram/*/wiki*.1gram.txt', cutoff=0, t0=t0)
    write_trie('akaza_data/data/system_language_model.1gram.trie', unigram)

    print(f"1gram. size={len(unigram)}")

    print(f'[{sys.argv[0]}] # 2gram')
    bigram = build_model('work/ngram/*/wiki*.2gram.txt', cutoff=3, t0=t0)
    write_trie('akaza_data/data/system_language_model.2gram.trie', bigram)

    # print(f'[{sys.argv[0]}] # 3gram')
    # trigram_dict, trigram = build_model('work/ngram/*/wiki*.3gram.txt', cutoff=100, t0=t0,
    #                                     prev_dict=bigram_dict)
    # write_trie('akaza_data/data/system_language_model.3gram.trie', trigram)

    print(f"[{sys.argv[0]}] 2gram. size={len(bigram)}")


def main():
    t0 = time.time()
    write_model()
    print(f"Elapsed: {time.time() - t0} seconds")


if __name__ == '__main__':
    main()
