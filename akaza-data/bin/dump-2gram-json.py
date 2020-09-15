import json

with open('jawiki.2gram.json', 'r') as fp:
    data = json.load(fp)

    for word1, word2data in data.items():
        total = sum(word2data.values())

        c2 = 0
        wc = 0

        for word2, count in word2data.items():
            c2 += count
            wc += 1

            # score = math.log10(count / total)

        print(f"{wc}\t{word1}")

