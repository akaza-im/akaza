import sys
import json

fname = sys.argv[1]

with open(fname, 'r') as fp:
    data = json.load(fp)
    for word1 in data:
        for word2 in data[word1]:
            for word3, score in data[word1][word2].items():
                print(f"{word1} {word2} {word3} {score}")
