import sys

# arpa n-gram ファイルの 1 gram データのエントリー数を数える。

fname = sys.argv[1]

count = 0

with open(fname, 'r') as fp:
    for line in fp:
        if line == "\\1-grams:\n":
            break

    for line in fp:
        if line == "\\2-grams:\n":
            break
        count += 1

print(count)
