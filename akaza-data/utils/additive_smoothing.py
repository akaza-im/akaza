import math

with open('work/jawiki.wfreq', 'r') as rfp:
    C = 0
    V = 0
    n = {}

    for line in rfp:
        m = line.split(' ')
        if len(m) == 2:
            word, cnt = m
            cnt = int(cnt)
            C += cnt
            V += 1
            n[word] = cnt

    alpha = 0.0000000001

    print(f"C={C}")
    print(f"V={V}")
    denominator = C + alpha*V
    print(f"C+αV={denominator}")
    print(f"default score:{math.log10(alpha / denominator)}")
    print(f"score for n=1:{math.log10((1+alpha)/denominator)}")
    print(f"score for n=10:{math.log10((10+alpha)/denominator)}")
    print(f"score for n=100:{math.log10((100+alpha)/denominator)}")
    print(f"score for n=1000:{math.log10((1000+alpha)/denominator)}")
