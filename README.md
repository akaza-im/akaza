# ibus-akaza

Yet another kana-kanji-converter on IBus, written in Rust.

統計的かな漢字変換による日本語IMEです。

**注意: もともと、Python で UI を書いて、 C++ でロジックを書いていたのですが、全面的に Rust で統一してみようかなと思って現在フルで書き直している途中です。ので、思ったより動かないです。2023/01
中には一通り UI が動くようにしたいです。**

## モチベーション

いじりやすくて **ある程度** UIが使いやすいかな漢字変換があったら面白いなと思ったので作ってみています。
「いじりやすくて」というのはつまり、Hack-able であるという意味です。

モデルデータを自分で生成できて、特定の企業に依存しない自由なかな漢字変換エンジンを作りたい。

## 特徴

* UI/Logic をすべて Rust で書いてあるので、拡張が容易です。
* 統計的かな漢字変換モデルを採用しています
    * 言語モデルの生成元は日本語 Wikipedia と青空文庫です。
        * 形態素解析器 Vibrato で分析した結果をもとに 2gram 言語モデルを構築しています。
        * 利用者の環境で 1 から言語モデルを再生成することが可能です。
* ユーザー環境で、利用者の変換結果を学習します(unigram, bigramの頻度を学習します)

## Dependencies

### Runtime dependencies

* ibus
* marisa-trie

### Build time dependencies

* rust

## Install 方法

ibus-akaza をインストールしてください。

    cd akaza-data/ && make && sudo make install
    cargo install --path ibus-akaza/
    sudo rustup stable
    cd ibus-akaza && make && sudo make install
    ibus restart

**注意**

akaza-data の生成には wikipedia の全データの展開および解析が走るので 1時間ぐらいはかかりますしディスク容量も 100GB ぐらいは余裕が必要です。
生成されたモデルデータを github actions で構築してファイルをリリースできるようにしたいと思っています。が、まだできてません。

## 設定方法

### config.yml

**この機能は現在未実装です**

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


## THANKS TO

* [ibus-uniemoji](https://github.com/salty-horse/ibus-uniemoji) を参考に初期の実装を行いました。
* [日本語入力を支える技術](https://gihyo.jp/book/2012/978-4-7741-4993-6) を読み込んで実装しました。この本がなかったら実装しようと思わなかったと思います。

