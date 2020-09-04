from comb import parse_skkdict
import marisa_trie

dictionary = parse_skkdict('/usr/share/skk/SKK-JISYO.L', encoding='euc-jp')

print("START")

t = []
for k, v in dictionary.items():
    vvv = '/'.join(v).encode('utf-8')
    t.append((k, vvv))

trie = marisa_trie.BytesTrie(t)

print("LOADED")


def gen_latice(s):
    for n in range(len(s) - 1):
        print(n)
        word = s[0:n]
        print(word)


src = 'ひつようなことは'
for prefix in reversed(trie.prefixes(src)):
    kanjis = trie[prefix][0].decode('utf-8').split('/')
    for kanji in kanjis:
        print(kanji + src[len(prefix):])
