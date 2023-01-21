# ibus-akaza

Yet another kana-kanji-converter on IBus, written in Rust.

統計的かな漢字変換による日本語IMEです。
Rust で書いています。

**現在、開発途中のプロダクトです。非互換の変更が予告なくはいります**

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

### Supported environment

* Linux 6.0 以上
* ibus 1.5 以上
* リトルエンディアン環境

## Install 方法

モデルファイルをダウンロードして展開してください。

    mkdir /usr/share/akaza/
    curl -L https://github.com/tokuhirom/akaza/releases/download/<<VERSION>>/akaza-data.tar.gz | tar xzv --strip-components=1 -C /usr/share/akaza/

ibus-akaza をインストールしてください。

    sudo rustup install stable
    sudo make install
    ibus restart
    ibus engine akaza

## 設定方法

### config.yml

XDG の設定ファイルディレクトリ以下、通常であれば `$HOME/.config/akaza/config.yml` に設定ファイルを書くことができます。

設定可能な項目は以下のもの。

* ユーザー辞書の設定

サンプルの設定は以下のような感じになります。
akaza が提供しているシステム辞書は偏りがすごくあるので、SKK-JISYO.L を読み込むことをおすすめします。たとえば以下のように設定すると良いでしょう。

    ---
    dicts:
      - path: /usr/share/skk/SKK-JISYO.L
        encoding: euc-jp
        dict_type: skk
      - path: /usr/share/skk/SKK-JISYO.jinmei
        encoding: euc-jp
        dict_type: skk
    single_term:
      - path: /usr/share/akaza/SKK-JISYO.dynamic
        encoding: utf-8
        dict_type: skk

akaza に付属する SKK-JISYO.dynamic を利用すると、「きょう」を変換すると、今日の日付がでるという機能が利用可能です。

↓ かな入力したい場合は以下のように設定してください。

    romkan: kana

### Keymap の設定


Akaza は典型的には以下の順番で探します。

1. `~/.local/share/akaza/keymap/{KEYMAP_NAME}.yml`
2. `/usr/local/share/akaza/keymap/{KEYMAP_NAME}.yml`
3. `/usr/share/akaza/keymap/{KEYMAP_NAME}.yml`

このパスは、[XDG ユーザーディレクトリ](https://wiki.archlinux.jp/index.php/XDG_%E3%83%A6%E3%83%BC%E3%82%B6%E3%83%BC%E3%83%87%E3%82%A3%E3%83%AC%E3%82%AF%E3%83%88%E3%83%AA) の仕様に基づいています。
Akaza は Keymap は `XDG_DATA_HOME` と `XDG_DATA_DIRS` からさがします。
`XDG_DATA_HOME` は設定していなければ `~/.local/share/` です。`XDGA_DATA_DIR` は設定していなければ `/usr/local/share:/usr/share/` です。

### RomKan の設定

ローマ字かなマップも同様のパスからさがします。

1. `~/.local/share/akaza/romkan/{KEYMAP_NAME}.yml`
2. `/usr/local/share/akaza/romkan/{KEYMAP_NAME}.yml`
3. `/usr/share/akaza/romkan/{KEYMAP_NAME}.yml`

### model の設定

model は複数のファイルからなります。

- unigram.model
- bigram.model
- SKK-JISYO.akaza

この切り替えは以下のようなところから読まれます。

- `~/.local/share/akaza/model/{MODEL_NAME}/unigram.model`
- `~/.local/share/akaza/model/{MODEL_NAME}/bigram.model`
- `~/.local/share/akaza/model/{MODEL_NAME}/SKK-JISYO.akaza`

keymap, romkan と同様に、`XDG_DATA_DIRS` から読むこともできます。

## FAQ

### 最近の言葉が変換できません/固有名詞が変換できません

流行り言葉が入力できない場合、[jawiki-kana-kanji-dict](https://github.com/tokuhirom/jawiki-kana-kanji-dict) の利用を検討してください。
Wikipedia から自動的に抽出されたデータを元に SKK 辞書を作成しています。
Github Actions で自動的に実行されているため、常に新鮮です。

一方で、自動抽出しているために変なワードも入っています。変なワードが登録されていることに気づいたら、github issues で報告してください。

### 人名が入力できません。など。

必要な SKK の辞書を読み込んでください。
現時点では config.yml を手で編集する必要があります。

https://skk-dev.github.io/dict/

## THANKS TO

* [ibus-uniemoji](https://github.com/salty-horse/ibus-uniemoji) を参考に初期の実装を行いました。
* [日本語入力を支える技術](https://gihyo.jp/book/2012/978-4-7741-4993-6) を読み込んで実装しました。この本がなかったら実装しようと思わなかったと思います。

