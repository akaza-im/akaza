import json

BIGRAM_CUTOFF = 10

with open('jawiki.2gram.json.orig', 'r') as fp:
    data = json.load(fp)

    for word1, word2data in data.items():
        total = sum(word2data.values())

        c2 = 0
        wc = 0

        for word2, count in word2data.items():
            if count <= BIGRAM_CUTOFF:
                continue
            c2 += count
            wc += 1

            # score = math.log10(count / total)

        print(f"{word1}\t{wc}\t{c2}")

