import os
import pathlib
import re
import sys
import jaconv

import MeCab
from janome.tokenizer import Tokenizer

t = Tokenizer()

targets = sys.argv[1:]
completed = 0


def get_token(node):
    if node.stat == MeCab.MECAB_BOS_NODE:
        return "<S>"
    elif node.stat == MeCab.MECAB_EOS_NODE:
        return "</S>"
    else:
        m = node.feature.split(',')
        if len(m) >= 8:
            yomi = node.feature.split(',')[7]
            return jaconv.kata2hira(yomi) + "/" + node.surface
        else:
            return node.surface


for ifilename in targets:
    ofilename = re.sub('^text/', 'dat/', ifilename)
    pathlib.Path(ofilename).parent.mkdir(parents=True, exist_ok=True)
    print(f"[{os.getpid()}] {ifilename} -> {ofilename} ({completed}/{len(targets)})")

    mecab = MeCab.Tagger('')
    mecab.parse('')  # free しすぎのやつのこと

    with open(ifilename, 'r') as rfp, \
            open(ofilename, 'w') as wfp:
        for line in rfp:
            if line.startswith('<') or len(line) == 0:
                continue

            tokens = []
            node = mecab.parseToNode(line)
            while node:
                token = get_token(node)
                tokens.append(token)
                node = node.next

            if len(tokens) > 2:
                wfp.write(' '.join(tokens) + "\n")

    completed += 1
