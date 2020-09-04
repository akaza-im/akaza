from comb import parse_skkdict
import marisa_trie

dictionary = parse_skkdict('/usr/share/skk/SKK-JISYO.L', encoding='euc-jp')

t = []
for k, v in dictionary.items():
    vvv = '/'.join(v).encode('utf-8')
    t.append((k, (vvv,)))

trie = marisa_trie.RecordTrie('@s', t)

print("LOADED")


def gen_latice(s):
    for n in range(len(s) - 1):
        print(n)
        word = s[0:n]
        print(word)


src = 'じゅうかきんぜい'
for s in trie.prefixes('たんげつ'):
    print(s)

# f = src[0]
# print(gen_latice(src))
