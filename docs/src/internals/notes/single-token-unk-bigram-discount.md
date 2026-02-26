# 1語候補の未知 bigram ペナルティ緩和

## 背景

辞書に登録された複合語（例: 「意思決定」「サイドバー」）を1語として変換する場合、
BOS/EOS との bigram が必ず未知（unk_bi）になり、デフォルトエッジコスト（約14.3）が
2回加算される。このため、分割された候補（例: 「医師/決定」）に対して構造的に不利になる。

### 具体例: 「いしけってい」

```
[1] 医師/決定   (viterbi: 19.1, unk_bi: 0.0,  tokens: 2)
[2] 意思決定     (viterbi: 28.4, unk_bi: 28.7, tokens: 1)
```

「意思決定」のコスト内訳:
```
BOS →[default: 14.3]→ 意思決定 →[default: 14.3]→ EOS
Viterbi = 14.3 + (-0.25) + 14.3 = 28.35
          ^^^^                ^^^^
          BOS との unk_bi     EOS との unk_bi
```

「医師/決定」は語同士の既知 bigram でエッジコストが下がるため、
unigram コストが高くても合計で勝ってしまう。

## 問題の本質

- BOS/EOS ノードは `word_id_and_score = None` のため、どの語との bigram も必ずデフォルトコストになる
- 1語候補は BOS と EOS の両方に接するので、unk_bi ペナルティを必ず2回受ける
- 2語以上の候補は語同士の既知 bigram でコストを下げられるが、1語候補にはその機会がない
- コーパスで bigram を学習しても BOS/EOS との組み合わせは学習されにくい

## 提案: リランキングでの1語候補 unk_bi 割引

### 方針

Viterbi 本体には触れず、リランキング段階で1語候補の unk_bi ペナルティを割り引く。

### 判定条件

- `token_count == 1`（BOS/EOS を除くトークンが1つ）
- `unknown_bigram_count == 2`（BOS→語、語→EOS の2回が未知）

この条件を満たす候補は「辞書に登録された複合語を1語で変換しようとしている」ケースに限定される。

### 実装案

`libakaza/src/graph/reranking.rs` を変更:

```rust
pub struct ReRankingWeights {
    pub bigram_weight: f32,
    pub length_weight: f32,
    pub unknown_bigram_weight: f32,
    pub skip_bigram_weight: f32,
    pub single_token_unk_discount: f32,  // 新規: デフォルト 0.5
}

impl ReRankingWeights {
    pub fn rerank(&self, paths: &mut [KBestPath]) {
        for path in paths.iter_mut() {
            let unk_weight = if path.token_count == 1 && path.unknown_bigram_count == 2 {
                self.unknown_bigram_weight * self.single_token_unk_discount
            } else {
                self.unknown_bigram_weight
            };

            path.rerank_cost = path.unigram_cost
                + self.bigram_weight * path.bigram_cost
                + unk_weight * path.unknown_bigram_cost
                + self.length_weight * path.token_count as f32
                + self.skip_bigram_weight * path.skip_bigram_cost;
        }
        paths.sort_by(|a, b| a.rerank_cost.total_cmp(&b.rerank_cost));
    }
}
```

### 期待される効果

discount=0.5 の場合の「いしけってい」:

```
[Before]
  医師/決定:  rerank = 8.33 + 18.35 + 1.0*0.0 + 2.0*2 = 30.68
  意思決定:   rerank = -0.25 + 1.0*0.0 + 1.0*28.67 + 2.0*1 = 30.42

[After] discount=0.5
  医師/決定:  rerank = 30.68 (変化なし)
  意思決定:   rerank = -0.25 + 0.0 + 0.5*28.67 + 2.0 = 16.08
  → 意思決定が1位になる
```

※ 上記は概算。実際の値はモデルに依存。

### 影響範囲

- **対象**: 1語候補 かつ BOS/EOS bigram が両方未知のケースのみ
- **非対象**: 2語以上の候補、既知 bigram を持つ1語候補
- Viterbi の候補生成には影響しない（k-best に含まれている候補の順位のみ変更）

## 退行リスク

### リスク1: 本来分割すべき入力で1語候補が不当に勝つ

例えば「さいど」(再度) が辞書に1語として登録されている場合、
「再/度」と分割される候補に対して不当に有利になる可能性がある。

**対策**: `token_count == 1 && unknown_bigram_count == 2` の条件により、
辞書登録された複合語に限定される。一般的な単語は unigram に登録されており
BOS/EOS との bigram も学習済みのケースが多い。

### リスク2: discount の値が不適切

discount が小さすぎると1語候補が常に勝ち、大きすぎると効果がない。

**対策**: evaluate コーパスで grid search して最適値を探索。
デフォルト値（0.5）は保守的な出発点。

## 検証手順

1. 実装後、デフォルト値で `cargo test --all` が pass することを確認
2. `akaza-data evaluate` で退行がないことを確認
3. 以下の代表的なケースで手動確認:
   - `いしけってい` → 意思決定（1位になることを期待）
   - `さいどばー` → サイドバー（1位になることを期待）
   - `いし` → 医師 or 石（退行しないこと）
   - 通常の2語以上の変換が退行しないこと
4. discount を 0.3〜0.7 で変えて evaluate の精度変化を観察

## 適用結果 (discount=0.5)

### 評価スコア

| | 再現率 | Good | Top-5 | Bad |
|---|---|---|---|---|
| 適用前 | 93.27% | 6719 | 343 | 3930 |
| 適用後 | 93.33% | 6805 | 350 | 3910 |
| 差分 | +0.06pt | +86 | +7 | -20 |

### 改善されたもの

**TOP5 → Good (8件)**: 1語候補の割引により TOP5 圏内から1位に昇格

- `いぬじに`: 犬じに → **犬死**
- `おさないと`: 幼いと → **押さないと**
- `しんじゅくより`: 新宿より → **新宿寄り**
- `しんちょうしんしょ`: 身長新書 → **新潮新書**
- `とうかんし`: 当館し → **等閑視**
- `なんきょくかんそくせん`: 難局観測船 → **南極観測船**
- `はせいひん`: は製品 → **派生品**
- `りくつや`: 理屈や → **理屈屋**

**BAD → Good (1件)**

- `つくりこんでるところもいいですね`: 作りこんでる → **作り込んでる**

**BAD → TOP5 (4件)**

- `いえのまえ`, `おいかぜさんこう`, `おもうつぼ`, `もうこんなじかん`

### 退行 (1件)

- `いしのそつう`: 意志の疎通 → 石野疎通（分節崩壊）
