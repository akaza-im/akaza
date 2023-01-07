# akaza の data flow

## language model の作成フロー

ここで、language model とは日本語における単語の発現確率のことを指す。

akaza では、基本的に wikipedia 日本語版のデータをもとに単語の発現確率及び 2gram での発現確率を求めている。

わかちがき処理及びよみがな処理には kytea を利用している。

wikipedia を利用しているのは、日本語のコーパスとしてフリーで再利用可能なものが他に見当たらないからであって、他に良いものがあれば追加したい。

```mermaid
graph TD
    wikipedia --> wikipedia.xml.bz2
    -- bunzip2 -->latest-pages-articles.xml
    -- wikiextractor --> extracted
    -- kytea --> annotated
    -- text2wfreq --> jawiki.wfreq
    -- wfreq2vocab --> jawiki.vocab
    jawiki.vocab --> dumpngram[/dumpngram/]
    extracted --> dumpngram[/dumpngram/]
    --> ngram.txt
    ngram.txt --> jawiki.mergeed-1gram.txt
    ngram.txt --> jawiki.mergeed-2gram.txt

    jawiki.mergeed-1gram.txt -- akaza-make-system-lm --> stats-kytea-lm_v2_1gram.trie
    jawiki.mergeed-2gram.txt -- akaza-make-system-lm --> stats-kytea-lm_v2_2gram.trie
    stats-kytea-lm_v2_1gram.trie -- akaza-make-system-lm --> stats-kytea-lm_v2_2gram.trie
```

## システム辞書

ひらがなと漢字の変換表として、システム辞書を用意している。
主に SKK の辞書と、wikipedia から生成された SKK 用の辞書である jawiki-kana-kanji-dict をベースにしている。

ここで生成される辞書形式は、BinaryDict と呼ばれている。

```mermaid
graph TD
    SKK-JISYO.L --> system-dict
    SKK-JISYO.jinmei --> system-dict
    SKK-JISYO.station --> system-dict
    jawiki-kana-kanji-dict --> SKK-JISYO.jawiki --> system-dict
    SKK-JISYO.akaza --> system-dict
    system-dict -- akaza-make-binary-dict--> stats-kytea-system_dict.trie

    SKK-JISYO.emoji --> single-term-dict
    SKK-JISYO.zipcode --> single-term-dict
    single-term-dict -- akaza-make-binary-dict--> stats-kytea-single_term.trie
```

## ユーザー言語モデル

akaza はユーザーごとに学習が可能なように設計されている。
シンプルに実装するために、ユーザー言語モデルはプレインテキスト形式で保存される。
プレインテキスト形式なので、ユーザーは自分の好きなようにファイルを変更することが可能である。

