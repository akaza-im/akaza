import glob
import sys
import re
import pathlib
import os
import MeCab

from janome.tokenizer import Tokenizer

t = Tokenizer()

targets = sys.argv[1:]
completed = 0

for ifilename in targets:
    ofilename = re.sub('^text/', 'dat/', ifilename)
    pathlib.Path(ofilename).parent.mkdir(parents=True, exist_ok=True)
    print(f"[{os.getpid()}] {ifilename} -> {ofilename} ({completed}/{len(targets)})")

    mecab = MeCab.Tagger('')
    mecab.parse('')

    with open(ifilename, 'r') as rfp, \
            open(ofilename, 'w') as wfp:
        for line in rfp:
            if line.startswith('<') or len(line) == 0:
                continue

            tokens = []
            n = mecab.parseToNode(line)
            while n:
                if n.stat == MeCab.MECAB_BOS_NODE:
                    tokens.append("<S>")
                elif n.stat == MeCab.MECAB_EOS_NODE:
                    tokens.append("</S>")
                else:
                    tokens.append(n.surface)
                n = n.next

            if len(tokens) > 2:
                wfp.write(' '.join(tokens) + "\n")

    completed += 1
