import sys
import re

spaces = re.compile(r'\s+')

wfreq = {}

fnames = sys.argv[1:]
completed = 0

for fname in fnames:
    sys.stderr.write(f"{completed}/{len(fnames)}\n")
    with open(fname, 'r') as fp:
        for line in fp:
            words = spaces.split(line)
            for word in words:
                if word not in wfreq:
                    wfreq[word] = 0
                wfreq[word] += 1
    completed += 1

for key, val in wfreq.items():
    print(f"{key} {val}")

