import sys
import json

fname = sys.argv[1]

with open(fname, 'r') as fp:
    data = json.load(fp)
    for first, trailing in data.items():
        for second, score in trailing.items():
            print(f"{first} {second} {score}")
