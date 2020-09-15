import os
import time
import glob

with open('jawiki.wfreq', 'r') as rfp, \
        open('jawiki.vocab', 'w') as wfp:
    vocab = []
    for line in rfp:
        m = line.rstrip().split(' ')
        if len(m) == 2:
            word, cnt = m
            if word.endswith('/UNK'):
                print(f"Skip: {word}")
                continue
            if int(cnt) > 15:
                vocab.append(word)
            else:
                print(f"Skip: {word}: {cnt}")

    for word in sorted(vocab):
        wfp.write(word + "\n")
