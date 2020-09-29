import marisa_trie
import random
import string
from timethese import cmpthese, pprint_cmp, timethese
import struct
import marisa


def get_random_string(length):
    letters = string.ascii_lowercase
    result_str = ''.join(random.choice(letters) for i in range(length))
    return result_str


data = [
    (get_random_string(8), random.random()) for n in range(1000000)
]

trie1 = marisa_trie.RecordTrie('<f', [(x[0], (float(x[1]),)) for x in data])
trie1.save('/tmp/xxx1')

trie2 = marisa_trie.BytesTrie([(x[0], struct.pack('<f', float(x[1]))) for x in data])
trie2.save('/tmp/xxx2')

trie3 = marisa_trie.BytesTrie([(x[0], struct.pack('<H', random.randint(0, 65535))) for x in data])
trie3.save('/tmp/xxx3')

# keyset = marisa.Keyset()
# for word, score in data:
#     keyset.push_back(word.encode('utf-8') + b"::" + struct.pack('<f', score))
# trie4 = marisa.Trie()
# trie4.build(keyset)


def test_trie1():
    trie1[data[532][0]][0][0]


def test_trie2():
    struct.unpack('<f', trie2[data[532][0]][0])[0]


def test_trie3():
    struct.unpack('<H', trie3[data[532][0]][0])[0]


cmp_res_dict = cmpthese(
    10000,
    {
        "trie1": test_trie1,
        "trie2": test_trie2,
        "trie3(int)": test_trie3,
    },
    repeat=300,
)
print(pprint_cmp(cmp_res_dict))
