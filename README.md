# ibus-comb

Yet another kana-kanji-converter on IBus, written in Python.

統計的かな漢字変換です。

## Dependencies

 * python-marisa
 * pip install wikiextractor

## 設計方針

 * モデル
  * モデルは 日本語 wikipedia 等から自動生成されて、誰でもチューニング可能なようにしたい。

## See also

 * [日本語入力を支える技術](https://gihyo.jp/book/2012/978-4-7741-4993-6)
 * http://www.phontron.com/slides/nlp-programming-ja-bonus-01-kkc.pdf
 * http://www.ar.media.kyoto-u.ac.jp/member/gologo/lm.html
 * [Japanese-Company-Lexicon](https://github.com/chakki-works/Japanese-Company-Lexicon)

## THANKS TO

I learned most of technique to implement ibus IME from ibus-uniemoji.

 * https://github.com/salty-horse/ibus-uniemoji
