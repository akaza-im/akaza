# ibus-comb

Yet another kana-kanji-converter on IBus, written in Python.

統計的かな漢字変換です。ibus の変換システムは Python でかけるので、かいてみました。

## 特徴

 * Python で書いてあるので、拡張が容易です。
 * 統計的かな漢字変換モデルを採用しています
 * 言語モデルの生成元は日本語 Wikipedia のみをベースとしています。
  * kytea で分析した結果をベースに 2gram 言語モデルを構築しています。

## Dependencies

 * python-marisa
 * pip install wikiextractor

## Install 方法

 * `make && sudo make install && ibus restart`

とかして、ibus 側の設定をすればOKです。

wikipedia の全データをダウンロードして言語モデルと辞書のロードが行われるために、ディスク容量とメモリと CPU がある程度必要です。
(TODO: ビルド済みモデルデータの配布)

## 設計方針

 * モデル
   * モデルは 日本語 wikipedia 等から自動生成されて、誰でもチューニング可能なようにしたい。
   * 現状、@tokuhirom は、 Wikipedia から生成された言語モデルで割と満足しています。
 * クローラーの提供
   * ユーザーが自分でクローラーを走らせることにより、言語モデルのトレーニングができるようにしたい。
 * ある程度 CPU/Memory が潤沢なシステムを想定しています。
   * メモリ8GB, CPU Intel core i5 程度を想定しています。
   * 現状は開発速度を優先しているために、メモリ容量を多めに消費するようになっています。
 * なにか面白い改善方法が思いついたら、fork して実装できるように。
   * 改造しやすい IME をめざす。
 * 品詞を扱わなくてもよいようにした

## See also

 * http://www.phontron.com/slides/nlp-programming-ja-bonus-01-kkc.pdf
 * http://www.ar.media.kyoto-u.ac.jp/member/gologo/lm.html
 * [Japanese-Company-Lexicon](https://github.com/chakki-works/Japanese-Company-Lexicon)

## THANKS TO

* [ibus-uniemoji](https://github.com/salty-horse/ibus-uniemoji) を参考に初期の実装を行いました。
* [日本語入力を支える技術](https://gihyo.jp/book/2012/978-4-7741-4993-6) を読み込んで実装しました。
