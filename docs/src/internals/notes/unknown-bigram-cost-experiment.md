# 未知 bigram コスト (Jelinek-Mercer スムージング) の実験

## 背景

未知 bigram のデフォルトコスト (~14.3) が重すぎて、辞書登録済み複合語（例: 各項目）が viterbi で分割パス（書く/項目）に負ける問題。Jelinek-Mercer スムージングの考え方に基づき、未知 bigram コストを下げることで解決を試みた。

## 実装

`LatticeGraph` に `unknown_bigram_cost` フィールドを追加し、`get_edge_cost_with_user_data()` / `get_edge_cost_detail_with_user_data()` / `get_default_edge_cost()` で、従来の `system_bigram_lm.get_default_edge_cost()` の代わりに使用する。

`GraphBuilder::new()` ではデフォルトで `system_bigram_lm.get_default_edge_cost()` を使用（既存動作と同等）。`with_unknown_bigram_cost()` で上書き可能。

`BigramWordViterbiEngineBuilder::build()` で環境変数 `AKAZA_UNKNOWN_BIGRAM_COST` が設定されていればその値を使用。

## 実験結果 (2026-03-02)

モデル: corpus-stats v2026.0216.0, commit 3fcf2b70

### Jelinek-Mercer 式: `-log₁₀(1 - λ)`

この公式では λ=0.7 でもコスト ≈ 0.52 にしかならず、元の ~14.3 に近い値を表現できない。実用的でない。

| λ | unknown bigram cost |
|---|---|
| 0.01 | 0.004 |
| 0.1 | 0.046 |
| 0.5 | 0.301 |
| 0.7 | 0.523 |
| 0.9 | 1.000 |

### 直接指定によるグリッドサーチ

| unknown_bigram_cost | Recall (%) | 退行幅 |
|---|---|---|
| 5 | 78.49 | -15.63 |
| 8 | 90.53 | -3.59 |
| 10 | 92.96 | -1.16 |
| 11 | 93.50 | -0.62 |
| 12 | 93.84 | -0.28 |
| 13 | 94.02 | -0.10 |
| 13.5 | 94.06 | -0.06 |
| 14 | 94.11 | -0.01 |
| ~14.3 (ベースライン) | **94.12** | 0.00 |

### 「各項目」変換テスト

全てのコスト値（0.004〜14.3）で「各項目」が1位。辞書に複合語として登録されているため、unknown bigram cost の値に依存しなかった。

## 「各項目」問題の実際の解決メカニズム

k-best の内訳を確認したところ、「各項目」は辞書複合語登録 + reranking の `single_token_unk_discount` で既に解決済みだった:

```
[1] 各項目 (viterbi: 38.05, rerank: 25.72, uni: 9.39, unk_bi: 28.67, unk_cnt: 2, tokens: 1)
[2] 各/項目 (viterbi: 22.87, rerank: 26.87, uni: 7.84, bi: 12.17, unk_cnt: 0, tokens: 2)
```

- Viterbi では「各/項目」(22.87) が「各項目」(38.05) に大差で勝つ
- Reranking で `single_token_unk_discount=0.5` が適用され、1語候補の unk_bi が半減
- 結果: 「各項目」(25.72) < 「各/項目」(26.87) で逆転

つまり unknown bigram cost 自体を変える必要はなかった。

## 結論

1. **Jelinek-Mercer 式 `-log₁₀(1-λ)` は不適切**。実用的なコスト範囲（10〜14）を λ で表現できない
2. **unknown bigram cost を下げると退行が増える**。コスト13以下では明確に退行が発生する
3. **「各項目」問題は辞書複合語登録 + reranking の `single_token_unk_discount` で既に解決済み**。unknown bigram cost の調整は不要だった

### 残した変更

- `LatticeGraph` に `unknown_bigram_cost` フィールドを追加（`GraphBuilder::with_unknown_bigram_cost()` で設定可能）
- デフォルトは `system_bigram_lm.get_default_edge_cost()` を使用（既存動作と完全互換）
- 将来的に別のスムージング手法を試す際の拡張点として機能する
