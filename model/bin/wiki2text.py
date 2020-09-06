import glob
import sys
import re
import pathlib
import os

from janome.tokenizer import Tokenizer

t = Tokenizer()

targets = sys.argv[1:]
completed = 0

for ifilename in targets:
    ofilename = re.sub('^text/', 'dat/', ifilename)
    pathlib.Path(ofilename).parent.mkdir(parents=True, exist_ok=True)
    print(f"[{os.getpid()}] {ifilename} -> {ofilename} ({completed}/{len(targets)})")

    with open(ifilename, 'r') as rfp, \
        open(ofilename, 'w') as wfp:
        for line in rfp:
            if line.startswith('<') or len(line) == 0:
                continue

            wfp.write(' '.join([token.surface for token in t.tokenize(line)]))

    completed += 1

