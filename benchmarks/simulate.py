import math

l = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
scores = []
for i in l:
    score = math.log10(i / 10)
    scores.append(score)

min_cost = min(scores)

print(f"min_cost={min_cost}")

for i, score in zip(l, scores):
    print(f"i={i}\tscore={score}\tnew={((score+(-min_cost))/(-min_cost))*65535}")
