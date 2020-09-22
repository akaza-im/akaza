# akaza-data

## What's this?

System dictionary/language model package for Akaza.

## PyPI's size limit

*The default size limit on PyPI is 60MB*

 * [unidic-lite](https://www.dampfkraft.com/code/distributing-large-files-with-pypi.html)

## Dependencies

 * wikiextractor

## How it works?

 1. 日本語版 wikipedia の jawiki-latest-pages-articles.xml.bz2 を取得
 2. bunzip2 で伸長
 3. wikiextractor で text/ 以下に展開する
 4. bin/wiki2text-runner.py で、kytea による読み推定を実施し、分かち書きして dat/ に保存。
 5. cat で jawiki.txt に連結して保存する。
 6. text2wfreq.py で頻度ファイルを生成する。 jawiki.wfreq
 7. wfreq2vocab.py で語彙ファイルを生成する。jawiki.vocab
   * ここの足切りラインを大きくすると変換精度は高まるが、生成データがでかくなる。
   * 現在は、「ペパボ」が入るという理由で 15 回以上登場したものとしている。

### system_language_model.trie

 a. bin/dumpngram.py で、vocab と text から、jawiki.2gram.json を生成する。
 b. json から system_language_model.trie を生成する。

### system_dict.trie

 a. system_dict.trie を jawiki.vocab から生成する。

## 生成されるデータ

### system_language_model.trie

marisa-trie 形式のデータです。1gram, 2gram のデータが素直に格納されています。

フォーマットは、2gram の場合は以下のようになっています。

  愛/あい\tは/は   => -0.525252

浮動小数点数がスコアです。このスコアは、n-gram の確率の log10 をとったものです。

## Size に関するメモ

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

