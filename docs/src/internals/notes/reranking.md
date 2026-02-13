# k-best リランキング機構

## 背景

現在の Viterbi アルゴリズムでは、パスコストが `Σ(unigram_cost + bigram_cost)` の等重み合算で決まる。
bigram コストが支配的になりやすく、レアな bigram の異常値に引きずられて誤変換が発生するケースがある。

例:
- たまたまコーパスに「は→厚い」の bigram が多い → 文脈に関係なく「厚い」が勝つ
- bigram は直前の 1 単語しか見ないため、「夏は暑い」vs「板は厚い」を区別できない

## 方針: 特徴量ベースの線形リランキング

Viterbi（等重み）で k-best 候補を生成した後、重み付きスコアで再順位付けを行う。

```
Viterbi (等重み) → k-best 候補生成（多様な候補の探索に最適化）
       ↓
ReRanking (重み付き) → 最終順位（最終選択に最適化）
```

候補生成と最終選択で最適な重みが異なるのは自然であり、
Viterbi 側は等重みのまま維持して多様な候補を確保する。

### 初期フェーズの特徴量

既存のコスト情報を分離し、パス長と未知 bigram を別特徴量として追加する:

```
rerank_score = 1.0 × Σ unigram_cost           (固定)
             + bigram_weight × Σ bigram_cost
             + length_weight × token_count
             + unknown_bigram_weight × unknown_bigram_cost_sum
```

- `unigram_weight`: **1.0 に固定**（基準スケールとして使い、他の重みを相対的に探索する）
- `bigram_weight`: デフォルト 1.0（= 従来と同じ挙動）
- `length_weight`: デフォルト 0.0（= パス長正規化なし）
- `unknown_bigram_weight`: デフォルト 1.0（= 通常の bigram と同じ扱い）

#### なぜ unigram_weight を固定するか

(unigram_weight, bigram_weight) はスケール不定で、
例えば (1.0, 0.7) と (10.0, 7.0) は同じ順位になる。
unigram を基準に固定することで探索空間を 1 次元減らし、グリッドサーチが安定する。

#### パス長正規化（length_weight）

`Σ cost` はトークン数に比例して増えるため、分節パターンが異なる候補が混ざると
短い分割が不当に有利になる副作用がある。
`length_weight` に正の値を入れると長い分割にボーナス、負の値でペナルティを制御できる。

#### 未知 bigram の分離（unknown_bigram_weight）

「レアな bigram 異常値」の多くは、実際には未知 bigram のフォールバック
（`default_edge_cost`）が原因。bigram 全体を弱めるより、未知 bigram だけを
別特徴量として切り出す方が副作用が少ない。
既知 bigram の判別力を維持しつつ、未知の暴れだけを抑制できる。

### スケールの事前確認

重み探索の前に、dev コーパスで以下を確認しておく:

- `Σ unigram_cost` と `Σ bigram_cost` の平均・分散
- パス長（トークン数）との相関
- 未知 bigram のフォールバック回数と寄与

bigram はエッジ数ぶん足されるため絶対値が大きくなりがちで、
unigram とスケールが大きく異なる場合がある。
この分布を把握しておくと、グリッドサーチの範囲が妥当になる。

### 将来の拡張

リランキングフレームワークが入れば、特徴量を追加するだけで拡張できる。
費用対効果を考慮した導入順:

| 順序 | 特徴量 | 効果 | 実装コスト |
|---|---|---|---|
| Phase 1 | unigram/bigram 分離 + len + unknown bigram | 基盤構築 + 未知ノイズ抑制 | 低 |
| Phase 2 | ルールベースペナルティ（特徴量として） | 既知の誤パターン抑制 | 低 |
| Phase 3 | Skip-gram 埋め込み | 離れた単語間の意味的整合性 | 高 |
| (将来) | Trigram スコア | 2 単語前まで文脈を拡張 | 中（コーパス規模が増えてから再検討） |

ルールペナルティは if/else で分岐するのではなく、特徴量として加点/減点する設計にする。
将来の重み学習と一貫性を保つため。

ルールペナルティの候補:
- 数字表記（漢数字/アラビア）不整合ペナルティ
- ひらがな連続が不自然（助詞崩壊）ペナルティ

#### Trigram vs Skip-gram の比較

| | Trigram | Skip-gram |
|---|---|---|
| 頻出パターンへの効果 | 高い | 高い |
| 未知の組み合わせへの汎化 | 弱い（スパース性の壁） | 強い（分散表現で汎化） |
| モデルサイズ | 大きくなりがち（bigram trie ~186MB の拡張） | 制御しやすい（語彙数 × 次元） |
| 既存コストとの統合 | 対数確率なので自然に加算 | 確率ではないが、線形モデルの特徴量としては問題なし |
| 実装の連鎖コスト | 高い（モデル構築・保存形式・検索・backoff 設計） | 中（学習は外部ツール可、推論は内積のみ） |

現在のコーパス規模（Wikipedia + 青空文庫）では trigram のスパース性が厳しいため、
skip-gram の方が費用対効果は高いと予想される。
trigram はコーパス規模が十分に増えた段階で再検討する。

## 設計

### KBestPath の拡張

`viterbi_cost`（候補生成で使った元のコスト）と `rerank_cost`（リランキング後のスコア）を
明確に分離する。`cost` フィールドの上書きは混乱の原因になるため避ける。

```rust
pub struct KBestPath {
    pub segments: Vec<Vec<Candidate>>,
    pub viterbi_cost: f32,              // Viterbi DP の合算コスト（変更しない）
    // リランキング用の特徴量内訳
    pub unigram_cost: f32,              // Σ unigram コスト
    pub bigram_cost: f32,               // Σ bigram コスト（既知 bigram のみ）
    pub unknown_bigram_cost: f32,       // Σ 未知 bigram のフォールバックコスト
    pub unknown_bigram_count: u32,      // 未知 bigram の回数
    pub token_count: u32,               // パス内のトークン数
    pub rerank_cost: f32,               // リランキング後のスコア（ソートキー）
}
```

### ReRankingWeights

```rust
pub struct ReRankingWeights {
    // unigram_weight は 1.0 固定（基準スケール）
    pub bigram_weight: f32,             // デフォルト 1.0
    pub length_weight: f32,             // デフォルト 0.0
    pub unknown_bigram_weight: f32,     // デフォルト 1.0
}

impl ReRankingWeights {
    pub fn rerank(&self, paths: &mut [KBestPath]) {
        for path in paths.iter_mut() {
            path.rerank_cost = path.unigram_cost
                + self.bigram_weight * path.bigram_cost
                + self.unknown_bigram_weight * path.unknown_bigram_cost
                + self.length_weight * path.token_count as f32;
        }
        paths.sort_by(|a, b| a.rerank_cost.partial_cmp(&b.rerank_cost).unwrap());
    }
}
```

デフォルト値 (bigram_weight=1.0, length_weight=0.0, unknown_bigram_weight=1.0) では
`rerank_cost = unigram_cost + bigram_cost + unknown_bigram_cost = viterbi_cost` となり、
従来と完全に同じ挙動になる。

### 重みの設定箇所

| 用途 | 設定方法 |
|---|---|
| `akaza-data check` | CLI: `--bigram-weight 0.7 --length-weight 0.1` 等 |
| `akaza-data evaluate` | CLI: 同上。グリッドサーチで最適値を探索可能 |
| ibus-akaza | `config.yml` の `engine.reranking_weights` |

### 変更対象ファイル

- `libakaza/src/graph/graph_resolver.rs` — KBestPath にコスト内訳追加、forward DP で分離記録
- `libakaza/src/graph/lattice_graph.rs` — edge cost 取得時に既知/未知を区別する情報を返す
- `libakaza/src/reranking.rs` (新規) — ReRankingWeights と rerank 関数
- `akaza-data/src/subcmd/check.rs` — CLI オプション追加
- `akaza-data/src/subcmd/evaluate.rs` — CLI オプション追加、評価メトリクス拡張
- `libakaza/src/config.rs` — EngineConfig に reranking_weights 追加

## 期待される効果

### ポジティブな効果

1. **未知 bigram ノイズの抑制**: unknown_bigram_weight を下げることで、未知 bigram のフォールバック異常値に引きずられにくくなる。既知 bigram の判別力は維持される
2. **一般的な単語の安定化**: bigram の重みが相対的に下がることで、unigram（単語自体の出現しやすさ）がアンカーとして機能し、変換結果が安定する
3. **パス長バイアスの制御**: length_weight により、短い分割が不当に有利になる副作用を補正できる
4. **チューニングの容易化**: evaluate コーパスに対してグリッドサーチで最適重みを探索できる。従来は bigram/unigram の比率を変えるにはモデル再構築が必要だった
5. **拡張の基盤**: 将来の特徴量追加（ルールペナルティ、skip-gram 等）が容易になる

### 定量的な期待

- `akaza-data evaluate` の exact match rate が数ポイント改善する可能性がある
- 特に未知 bigram が多い短い入力（2〜3 文節）で効果が出やすい
- 既知 bigram カバレッジが高い頻出パターンでは従来と同等の精度を維持

## 退行リスク

### リスク1: bigram を弱めすぎると同音異義語の判別力が低下

bigram は同音異義語の判別に本質的に効いている（例: 「板が厚い」vs「お湯が熱い」）。
bigram_weight を下げすぎると、直前の文脈が効かなくなり、これらのケースで退行する。

**対策**: evaluate コーパスの must.txt / should.txt で退行検知。デフォルト重みでは従来と完全に同じ挙動を保証。

### リスク2: Viterbi の候補生成と最終順位の乖離

Viterbi は等重みで候補を生成するため、リランキング後に「本来 1 位になるべきパスが k-best に含まれていない」可能性がある。

**対策**:
- k の値を十分大きくする（5〜10）
- evaluate で **top-k hit rate** を常に出力し、「rerank の改善余地が候補生成で潰れていないか」を数値で追跡
- 必要に応じて候補生成側の多様化（unknown bigram フォールバックのクリップ等）

### リスク3: 重みの過学習

evaluate コーパスに過度にフィットした重みは、汎用的な変換で退行する可能性がある。

**対策**: コーパスを train/dev に分割して交差検証。極端な重み（0.0 や 10.0 等）にならないよう範囲を制限。

### リスク4: パフォーマンスへの影響

リランキング自体は k 個のパスを再ソートするだけなので計算コストはほぼゼロ。
KBestPath にフィールドが増えるが、f32 数個の追加のみで影響は無視できる。

## 検証手順

### 実装フェーズ

安全のため段階的に進める:

1. **Phase 1**: コスト分離のみ。KBestPath に viterbi_cost/rerank_cost/特徴量内訳を追加。デフォルトでは rerank_cost = viterbi_cost で完全一致を確認
2. **Phase 2**: CLI/config でリランキング重みを設定可能にし、rerank を有効化
3. **Phase 3**: evaluate に top-k hit rate、must/should の差分レポートを追加
4. **Phase 4**: length_weight と unknown_bigram_weight を特徴量として追加（ここで精度が動きやすい）

### 評価メトリクス

重み探索時には以下を同時に確認する:

- **top-1 accuracy** (exact match rate)
- **top-k hit rate** (k=5, k=10)
- **must.txt の退行数** — 0 であること
- **should.txt の改善数/退行数**
- **LCS-based recall** (既存メトリクス)

### テスト手順

1. `cargo test --all` で既存テストが pass することを確認
2. デフォルト重みで `akaza-data evaluate` の結果が従来と完全に一致することを確認
3. dev コーパスで `Σ unigram_cost`、`Σ bigram_cost`、`unknown_bigram_count` の分布を確認
4. bigram_weight を 0.5〜0.9 で変えながら evaluate を実行し、精度の変化を観察
5. must.txt の全ケースで退行がないことを確認
6. `akaza-data check` で代表的な入力を手動確認
