from comb.engine import parse_skkdict
import pygtrie

dictionary = parse_skkdict('/usr/share/skk/SKK-JISYO.L', encoding='euc-jp')

t = pygtrie.CharTrie()

for k, v in dictionary.items():
    vvv = '/'.join(v).encode('utf-8')
    t[k] = v

print("LOADED")


def gen_latice(s):
    for n in range(len(s) - 1):
        print(n)
        word = s[0:n]
        print(word)


src = 'じゅうかきんぜい'
# print(t.get('じゅうか'))
for s in t.prefixes('たんげつ'):
    print(s)

# f = src[0]
# print(gen_latice(src))
