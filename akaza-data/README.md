# akaza-data

## What's this?

System dictionary/language model package for Akaza.

## PyPI's size limit

*The default size limit on PyPI is 60MB*

 * [unidic-lite](https://www.dampfkraft.com/code/distributing-large-files-with-pypi.html)

## Size

 * word1 + word2 + score
 * 4byte + 4byte + 2byte

entries(bigram cutoff=3):

    1gram:   297,228

bigram entries:

 -  3: 5,744,624
 - 10: 2,639,415
 - 20: 1,603,540
 - 50:   803,462

5M * 10 = 50MB

## See also

