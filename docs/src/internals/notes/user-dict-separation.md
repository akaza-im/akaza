# ユーザー辞書の自動学習データ分離

## 現状の問題

### SKK-JISYO.user の二重用途

`~/.local/share/akaza/SKK-JISYO.user` が 2 つの異なる用途で使われている:

1. **compound_word 自動学習の保存先**: `UserData.dict` に蓄積された複合語学習データを、バックグラウンドスレッド（3 秒間隔）で `write_user_files()` により書き出す
2. **ユーザー手動編集の辞書**: `find_user_dicts()` で `userdict/` ディレクトリが空の場合のフォールバックとして、IBus メニューの「ユーザー辞書」に表示され、`akaza-dict` で手動編集される

### 具体的な問題

#### 問題 1: akaza-dict との書き込み競合

1. ユーザーが IBus メニューから `akaza-dict` を起動し、`SKK-JISYO.user` を手動編集・保存
2. 直後に `ibus-akaza` のバックグラウンドスレッドが `write_user_files()` を実行
3. メモリ上の `self.dict`（起動時に読み込んだ古い内容 + 自動学習分）で上書きされ、手動編集の変更が消失

#### 問題 2: load 失敗時のデータ消失

1. `UserData::load_from_default_path()` が何らかの理由で失敗
2. `UserData::default()`（dict が空）でフォールバック起動
3. バックグラウンドスレッドが空の dict で `SKK-JISYO.user` を上書き
4. 手動登録した単語がすべて消失

#### 問題 3: 再読み込みの欠如

`ibus-akaza` は `SKK-JISYO.user` を起動時に一度だけ読み込む。`akaza-dict` が別プロセスで辞書を変更しても、`ibus-akaza` は変更を検知・反映しない。

## 現状のデータフロー

```
起動時:
  SKK-JISYO.user → read_skkdict() → UserData.dict (メモリ)

変換確定時:
  compound_word == true の候補 → UserData.dict に追加

3秒ごと:
  UserData.dict → write_skk_dict() → SKK-JISYO.user (上書き)

akaza-dict (別プロセス):
  SKK-JISYO.user → 読み込み → GUI 編集 → 保存 → SKK-JISYO.user (上書き)
  ※ ibus-akaza と排他制御なし
```

## 変更後のデータフロー

```
起動時:
  compound_dict.v2.bin → bincode + 復号 → UserData.dict (メモリ)
  SKK-JISYO.user は UserData では読み込まない (変換エンジンが辞書として読む)

変換確定時:
  compound_word == true の候補 → UserData.dict に追加 (変更なし)

3秒ごと:
  UserData.dict → bincode + 暗号化 → compound_dict.v2.bin
  ※ SKK-JISYO.user には触らない

akaza-dict (別プロセス):
  SKK-JISYO.user → 読み込み → GUI 編集 → 保存 → SKK-JISYO.user
  ※ ibus-akaza の write_user_files と競合しなくなる
```

## 実装方針

### compound_word 学習データをバイナリ形式に分離

compound_word の自動学習データは:

- ユーザーが手で編集する必要がない
- unigram/bigram/skip_bigram と同じ性質のデータ（自動学習・自動保存）
- 既に unigram/bigram/skip_bigram は v2 暗号化バイナリ形式に移行済み

したがって、`self.dict` の保存先を SKK テキスト形式から v2 バイナリ形式に変更する。

### 変更内容

1. **保存先の変更**: `SKK-JISYO.user` → `compound_dict.v2.bin`（暗号化バイナリ）
2. **write_user_files() から SKK-JISYO.user への書き込みを削除**
3. **読み込み**: 起動時に `compound_dict.v2.bin` から `self.dict` を復元
4. **既存の v1 テキスト形式**: 鍵なしの場合のフォールバックとして `compound_dict.v1.txt` も用意（unigram 等と同じパターン）

### マイグレーション

既存の `SKK-JISYO.user` に含まれる compound_word データのマイグレーションは不要。compound_word は日常的な変換操作で再学習されるため、自然に復元される。

一方、ユーザーが手動登録した単語は `SKK-JISYO.user` にそのまま残り、`write_user_files()` が触らなくなるため安全に保持される。

### 副次的な改善

- `find_user_dicts()` のフォールバックで `SKK-JISYO.user` を返す動作は、自動学習データとの混在がなくなるため安全になる
- 将来的に `find_user_dicts()` を改善して、初回起動時に `userdict/` に明示的なユーザー辞書を作成する方向も検討可能

## 関連コード

| ファイル | 関連箇所 |
|---------|---------|
| `libakaza/src/user_side_data/user_data.rs` | `UserData.dict`, `write_user_files()`, `record_entries()`, `load()` |
| `libakaza/src/dict/skk/write.rs` | `write_skk_dict()` |
| `libakaza/src/dict/skk/read.rs` | `read_skkdict()` |
| `libakaza/src/graph/graph_builder.rs` | `user_data.dict.get(yomi)` で候補追加 |
| `ibus-akaza/src/main.rs` | バックグラウンド保存スレッド（3 秒間隔） |
| `ibus-akaza/src/ui/prop_controller.rs` | `find_user_dicts()`, ユーザー辞書メニュー |
| `akaza-dict/src/conf.rs` | 辞書編集 GUI |
