# akaza-data

## What's this?

System dictionary/language model package for Akaza.

## How to build this?

    make
    make install

## Dependencies

* wikiextractor
* python3
* rust
* wget

## How it works?

TODO: 書き直し

1. 日本語版 wikipedia の jawiki-latest-pages-articles.xml.bz2 を取得
2. bunzip2 で伸長
3. wikiextractor で text/ 以下に展開する
4. bin/wiki2text-runner.py で、kytea による読み推定を実施し、分かち書きして dat/ に保存。
5. cat で jawiki.txt に連結して保存する。
6. text2wfreq.py で頻度ファイルを生成する。 jawiki.wfreq
7. wfreq2vocab.py で語彙ファイルを生成する。jawiki.vocab

* ここの足切りラインを大きくすると変換精度は高まるが、生成データがでかくなる。
* 現在は、「ペパボ」が入るという理由で 16 回以上登場したものとしている。

### system_language_model.trie

a. bin/dumpngram.py で、vocab と text から、jawiki.1gram.json/jawiki.2gram.json を生成する。
b. json から system_language_model.trie を生成する。

### system_dict.trie

a. system_dict.trie を jawiki.vocab から生成する。

## 生成されるデータ

### system_language_model.trie

marisa-trie 形式のデータです。1gram, 2gram のデータが素直に格納されています。

フォーマットは、2gram の場合は以下のようになっています。

愛/あい\tは/は => -0.525252

浮動小数点数がスコアです。このスコアは、n-gram の確率の log10 をとって - をつけたものです。

## Size に関するメモ

以下でざっくりとした見積もりが書いてあるが、現実的にはトライ構造で圧縮されるため、その通りにはならないです。

* word1 + word2 + score
* 4byte + 4byte + 2byte

entries(bigram cutoff=3):

    1gram:   297,228

bigram entries:

- 3: 5,744,624
- 10: 2,639,415
- 20: 1,603,540
- 50:   803,462

5M * 10 = 50MB

## 調整方法

誤変換が多いな、と思ったら。

* vibrato がトーカナイズできてなくてスコアがついてないな、と思った場合は mecab-user-dict.csv にエントリーを追加してください。
    * akaza-data/ で make all したあとに、該当単語が work/jawiki/vibrato-ipadic.vocab に入っていなければ、mecab-user-dict.csv
      に追加する必要があります
    * このファイルには、明らかに必要な元号/国名などのみを追加してください。
    * 固有名詞などは、入れたとて Wikipedia 内での記述回数が少ないために、スコアがつかないので無意味です。

## LICENSE

生成データには GPL の SKK の辞書が含まれているので、GPL 扱い。
これは、今後 SudachiDic か IPADIC か UniDic か何かをベースにするように変更するかもしれない。

それ以外の Rust で書かれたコードベースなどは MIT License.

## See also

