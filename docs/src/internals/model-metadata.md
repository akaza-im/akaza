# モデルファイルのメタデータ

Akaza のモデルファイルにはビルド時のバージョン情報が埋め込まれている。
これにより、インストール済みモデルのバージョンやビルド日時を確認できる。

## メタデータ項目

| キー | 説明 | 例 |
|------|------|-----|
| `AKAZA_DATA_VERSION` | ビルドに使用した akaza-data のバージョン | `0.1.7` |
| `BUILD_TIMESTAMP` | ビルド日時（UTC, ISO 8601） | `2026-02-20T12:34:56Z` |

## 確認方法

`akaza-data model-info` コマンドでメタデータを表示できる。

```bash
# marisa-trie 形式のモデルファイル
akaza-data model-info data/unigram.model
# File: data/unigram.model
# Type: marisa-trie
# Keys: 1058884
# AKAZA_DATA_VERSION: 0.1.7
# BUILD_TIMESTAMP: 2026-02-20T12:34:56Z

# bigram, skip_bigram も同様
akaza-data model-info data/bigram.model
akaza-data model-info data/skip_bigram.model

# SKK 辞書ファイル（テキスト形式）
akaza-data model-info data/SKK-JISYO.akaza
# File: data/SKK-JISYO.akaza
# Type: skk-dict
# AKAZA_DATA_VERSION: 0.1.7
# BUILD_TIMESTAMP: 2026-02-20T12:34:56Z
```

## 格納形式

### marisa-trie 形式（.model ファイル）

既存のメタデータキー（`__TOTAL_WORDS__`、`__DEFAULT_EDGE_COST__` 等）と同じパターンで、
特殊キーとして trie に格納される。

```
__BUILD_TIMESTAMP__\t2026-02-20T12:34:56Z
__AKAZA_DATA_VERSION__\t0.1.7
```

対象ファイル:
- `unigram.model`
- `bigram.model`
- `skip_bigram.model`

### テキスト形式（SKK-JISYO.akaza）

SKK 辞書フォーマットのコメント行（`;;` で始まる行）としてファイル先頭に記録される。

```
;; AKAZA_DATA_VERSION: 0.1.7
;; BUILD_TIMESTAMP: 2026-02-20T12:34:56Z
;; okuri-ari entries.
;; okuri-nasi entries.
...
```

## 後方互換性

メタデータが存在しない古いモデルファイルでも正常に動作する。
`model-info` コマンドではメタデータがない場合 `(not set)` と表示される。
