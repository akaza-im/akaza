# TODO

## Priority high

- 単語 2 gram を学習データとして記録できるようにする
- 末尾のアルファベット一文字は、変換しない。
- [BUG] 

## Priority mid

- 共起的なスコアをいれたい?
- support 3gram(必要?)
- 青空文庫をコーパスとして使う?
- 絵文字辞書(`ビール` か `beer` で絵文字いれたい。)
- 平仮名語辞書もいるのかもしれない。

- 変な変換
  - "げすとだけ" が "げストだけ" になる。
  - "bきゅう" が "B級" にならない
  - "ぜんぶでてるやつ" -> 全部で輝やつ"
  - "ERABERU" -> "鰓ベル"
  - "NIHONN"
  - "難し区内"
  - "SOUMITAIDESUNE"
  - "DOGGUFU-DHINGUSIDURAI"
  - "PEPABO"
  - "HAYAKUCHI"
  - "KOUIU"
  - "KIZON"
  - "NIHON"
  - "NOZOMASII"

## Priority low

- 設定画面の実装
- 単語登録機能
- 前向きDP後ろ向きA* で候補を得る

# DONE

- 「きょう」を %Y-%m-%d 等に変換できるようにしたい。
- カタカナ語辞書の作成
- 連文節変換用の UI を実装する
- Function key とかのショートカットで、全部カタカナにすることができるように。
  - ibus-comb がバグってた時に便利。
