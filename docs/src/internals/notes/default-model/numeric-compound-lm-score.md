# 数値複合語の LM スコア選択改善

## 問題

graph_builder.rs の数値複合語パスで `find().or_else(normalize)` パターンを使用していたため、
リテラルマッチが見つかった時点で正規化版を試さず、常にリテラル側のスコアが使われていた。

例: `"1日/1にち"`
- リテラルマッチ: score=9.386（悪い）
- `<NUM>日/<NUM>にち` 正規化版: score=2.123（良い）

`or_else` のセマンティクスにより、リテラルが存在すれば正規化版は検索されない。
結果として「1日」のスコアが「1日祭」より悪くなり、Viterbi が誤ったパスを選択していた。

## 修正

数値複合語ブロック内の LM lookup で、リテラルと `<NUM>` 正規化版の両方を検索し、
スコアが良い方（値が小さい方）を採用するように変更。

```rust
// Before: or_else でリテラル優先
let word_id_and_score =
    self.system_unigram_lm.find(&key_buf).or_else(|| {
        normalize_surface_for_lm(&key_buf)
            .and_then(|nk| self.system_unigram_lm.find(&nk))
    });

// After: 両方検索してスコアが良い方を採用
let word_id_and_score = {
    let direct = self.system_unigram_lm.find(&key_buf);
    let normalized = normalize_surface_for_lm(&key_buf)
        .and_then(|nk| self.system_unigram_lm.find(&nk));
    match (direct, normalized) {
        (Some(d), Some(n)) => Some(if d.1 <= n.1 { d } else { n }),
        (d @ Some(_), None) => d,
        (None, n @ Some(_)) => n,
        (None, None) => None,
    }
};
```

## 影響範囲

- 数値複合語（数字+助数詞）のスコアリングのみ
- 一般辞書引き（L119, L145）には影響しない
- `<NUM>` 正規化エントリが LM にある場合のみ動作が変わる

## テスト

- `test_numeric_compound_picks_better_score`: 正規化版のスコアが良い場合に正規化版が採用されることを確認
- `test_numeric_compound_keeps_better_literal_score`: リテラルのスコアが良い場合にリテラルが採用されることを確認

## 評価結果

corpus-stats v2026.0216.0 での anthy-corpus 評価結果:

| | 変更前 | 変更後 | 差分 |
|---|---|---|---|
| Original BAD | 3993 | 3996 | +3 |
| Accepted (style/OK) | 507 | 512 | +5 |
| **Real BAD** | **3473** | **3471** | **-2** |
| 再現率 | 93.186% | 93.186% | ±0 |

### 改善例

- `1にちさいちょう8じかん`: `1日祭超8字管` → `１日最長８時間`（分節崩壊解消）
- `5ふん`: `5扮` → `５分`（助数詞の誤変換解消）
- `3ねん`: `3念` → `３年`（助数詞の誤変換解消）
- `50ねん`: `50念` → `５０年`（助数詞の誤変換解消）
- `1にちも`: `1にちも`（未変換）→ `１日も`（変換成功）

### 副作用: 半角→全角数字の表記変化

`<NUM>` 正規化版のスコアが採用されることで、全角数字バリアントが優先されるケースが発生。
これは数値複合語の3バリアント（半角/全角/漢数字）に同じスコアが割り当てられ、
ノードの登録順で全角が先に選ばれるため。

- `1日` → `１日`、`5分` → `５分` 等

これらは表記スタイルの差であり、accept.tsv で許容した（5件追加）。
