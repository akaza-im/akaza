# TODO

## Priority high

- CI を実施する

## Priority mid

- support 3gram(必要?)
- ユーザー言語モデル学習機 from text file or web.
  - クローラーをかく？
- ユーザー辞書機能を実装する
- packaging to arch linux.
- キーバインディングを設定可能に。
- 設定候補の最後にアルファベットなどを入れる。

- 変な変換
  - ドンだけと"bきゅう" が "B級" にならない
  - "どんだけとちかんなくてもわかりそうなもんだよねw"

- release the package to github.

## Priority low

- 文節として、一文字の平仮名のみになったものに関しては、連結してしまった方が良いのではないだろうか？
- when user edit user dictionary, reload it
- 設定画面の実装
- 単語登録機能
- かな入力サポート

## priority very low

- fcitx 対応(python 以外で書き直さないと難しそう)
- When user language model is too big, purge data automatically.
  - if the data is bigger than 1GB?
- NICOLA support?
- 共起的なスコアをいれたい?
- 青空文庫をコーパスとして使う?
  - 古くさすぎるかも
- 言語モデルを小さくできないか?

# DONE

- 「きょう」を %Y-%m-%d 等に変換できるようにしたい。
- カタカナ語辞書の作成
- 連文節変換用の UI を実装する
- Function key とかのショートカットで、全部カタカナにすることができるように。
  - ibus-akaza がバグってた時に便利。
- 末尾のアルファベット一文字は、変換しない。
- 前向きDP後ろ向きA* で候補を得る
- 平仮名語辞書もいるのかもしれない。
- 分節の区切り箇所を自分で指定できるようにしたい。
- 単語 2 gram を学習データとして記録できるようにする
- 絵文字辞書(`ビール` か `beer` で絵文字いれたい。)
- ベースのシステム辞書は、marisa-trie 形式でビルドしたものをロードするようにした法が良いのではないか?
