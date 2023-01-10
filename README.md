# ibus-akaza

Yet another kana-kanji-converter on IBus, written in Python.

統計的かな漢字変換です。そのうち、機械学習に切り替えるかも。未定。

**注意: もともと、Python で UI を書いて、 C++ でロジックを書いていたのですが、全面的に Rust で統一してみようかなと思って現在フルで書き直している途中です。ので、思ったより動かないです。2023/01 中には一通り UI が動くようにしたいです。**

## モチベーション

いじりやすくてある程度UIが使いやすいかな漢字変換があったら面白いなと思ったので作ってみています。
「いじりやすくて」というのはつまり、Hack-able であるという意味です。

kkc はモデルデータを自分で再構築することができない。気がします。
あと、vala を学ぶのがしんどい。。

## 特徴

 * UI/Logic を Rust で書いてあるので、拡張が容易です。
 * 統計的かな漢字変換モデルを採用しています
   * 言語モデルの生成元は日本語 Wikipedia のみをベースとしています。
     * 形態素解析器 Vibrato で分析した結果をベースに 2gram 言語モデルを構築しています。
     * 利用者の環境で、1から言語モデルを再生成することが可能です。
     * そこそこのマシンパワーとディスク容量を必要とします。
 * ユーザー環境で、利用者の変換結果を学習します(unigram, bigramのスコアを学習します)

## Dependencies

 * ibus
 * marisa-trie

## Install 方法

    cd akaza-data/ && pip install -r requirements.txt
    cd akaza-core/ && cargo install
    make && sudo make install && ibus restart

ibus 側の設定をすればOKです。

## 設定方法

### config.yml

XDG の設定ファイルディレクトリ以下、通常であれば `$HOME/.config/ibus-akaza/config.yml` に設定ファイルを書くことができます。

設定可能な項目は以下のもの。

 * ローマ字変換テーブルの上書き
 * ユーザー辞書の設定

サンプルの設定は以下のような感じになります。

    # ライブ変換モード
    live_conversion: True
    romaji:        # ローマ字変換テーブル
      la: ら
    user_dicts:    # ユーザー辞書の設定
      - path: /home/tokuhirom/dotfiles/skk/SKK-JISYO.tokuhirom
        encoding: utf-8

## 設計方針

 * モデル
   * モデルは 日本語 wikipedia 等から自動生成されて、誰でもチューニング可能なようにしたい。
   * 現状、@tokuhirom は、 Wikipedia から生成された言語モデルで割と満足しています。
   * anthy よりも変換精度が高い気がしています
 * クローラーの提供
   * (未実装) ユーザーが自分でクローラーを走らせることにより、言語モデルのトレーニングができるようにしたい。
 * なにか面白い改善方法が思いついたら、fork して実装できるように。
   * 改造しやすい IME をめざす。
 * 辞書のメンテや実装において、品詞を扱わなくてもよいようにした

## Roadmap

 * 一通り開発者自身が使っていて違和感がないようなレベルまで持っていく
 * コスト計算のチューニングを可能なようにする
  * 統計的かな漢字変換をベースにしていくのか、構造化SVM ないし 構造化パーセプトロンベースにしていくのか
  * あるいは RNN を採用するのか、他の方法で共起コストを入れるのか
 * ドキュメントを整理する

## See also

 * http://www.phontron.com/slides/nlp-programming-ja-bonus-01-kkc.pdf
 * http://www.ar.media.kyoto-u.ac.jp/member/gologo/lm.html
 * [Japanese-Company-Lexicon](https://github.com/chakki-works/Japanese-Company-Lexicon)
 * [Faster and smaller n-gram language models](https://www.aclweb.org/anthology/P11-1027.pdf)
 * https://qiita.com/nekoaddict/items/ba4cfb972da6886cf1be
 * https://github.com/lv7777/ibus-levena/

## THANKS TO

* [ibus-uniemoji](https://github.com/salty-horse/ibus-uniemoji) を参考に初期の実装を行いました。
* [日本語入力を支える技術](https://gihyo.jp/book/2012/978-4-7741-4993-6) を読み込んで実装しました。
