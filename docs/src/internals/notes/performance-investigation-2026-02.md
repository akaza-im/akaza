# 性能調査ノート (2026-02-25)

## ベースライン (release build, akaza-data bench 50文)

```
avg=46.1ms, median=27.3ms, p95=164.5ms, p99=267.2ms, max=423.2ms
```

長文ほど重い。25文字の「あさはやくおきないとせいしんえいせいじょうよくない」で max=423ms。

## perf プロファイル結果

`perf record -g --call-graph dwarf -F 997` で取得。

### CPU ホットスポット (self time 上位、変換時)

| self % | 関数 | 分類 |
|--------|------|------|
| 13.0% | `LoudsTrie::predictive_search` | trie 検索ループ |
| 9.7% | `LoudsTrie::prefix_match_` | trie 前方一致 |
| 4.6% | `SkipBigramUserStats::get_cost` | skip-bigram ユーザー統計 |
| 4.3% | `GraphResolver::resolve_k_best` | Viterbi DP 本体 |
| 3.4% | `normalize_counter_key_for_lm` | 数字正規化 |
| 3.3% | `sort_impl` (rsmarisa build 時) | trie 構築 |
| 3.3% | `LoudsTrie::restore_` | trie キー復元 |
| 3.0% | `BitVector::select1` | LOUDS ビット操作 |
| 0.8% | `BitVector::select0` | LOUDS ビット操作 |
| 0.4% | `BitVector::rank1` | LOUDS ビット操作 |
| 6.4% | `_int_free` (libc) | メモリ解放 |
| 4.1% | `__memmove_avx` (libc) | メモリコピー |
| 1.4% | `str::join_generic_copy` | 文字列結合 |

### ボトルネック分類

1. **rsmarisa trie 操作 ~30%** — predictive_search + prefix_match_ + restore_ + select + rank 等
   - 呼び出し元は **skip-bigram LM が最大** (コールスタックで確認)
   - 次いで bigram LM、辞書検索
2. **メモリ alloc/free ~12%** — _int_free + malloc + memmove + realloc
3. **数字正規化 ~5%** — normalize_counter_key_for_lm + parse 関数群

## 現状のデータ構造の使われ方

重要な発見: **全 LM とも `predictive_search` を呼んでいるが、実態はほぼ exact match**。
キーの末尾にスコアがバイナリ埋め込みされており、結果は 1 つしか使わない。

| コンポーネント | サイズ | キー形式 | 実際の操作 |
|---|---|---|---|
| Unigram LM | 16 MB | `"漢字/かな\xff<4B f32>"` | 事実上 exact match |
| Bigram LM | 69 MB | `[3B id1][3B id2][2B f16]` (固定 8B) | 事実上 exact match |
| Skip-bigram LM | 114 MB | `[3B id1][3B id2][2B f16]` (固定 8B) | 事実上 exact match |
| 辞書 | 43 MB | `"かな\t候補1/候補2/..."` | prefix search（本当に必要） |

毎回の検索で `Agent::new()` → `Box<State>` アロケーション → drop を行っている。再利用なし。

### 参考: Mozc のサイズ

```
mozc_server バイナリ: 18 MB
  .rodata (辞書+LMデータ): 16.4 MB
  .text (コード): 1.0 MB
```

Akaza の辞書+LM 合計 228 MB と比較して桁違いに小さい。

## rsmarisa 内部のホットスポット詳細

perf の関数レベル内訳から、rsmarisa 内で何に時間がかかっているかを分析。

### predictive_search (13.0%)

`predictive_search` は Akaza が bigram/skip-bigram の exact match に使っている主関数。
内部で以下を呼ぶ:

1. **`predictive_find_child`** — クエリの各バイトについて子ノードを探索
   - キャッシュヒット時: `cache[cache_id].parent() == node_id` → `prefix_match` or 直接遷移
   - キャッシュミス時: `louds.select0(node_id) + 1` → 子ノードを線形探索

2. **`prefix_match_` (9.7%)** — link ノード（再帰 trie の下位レベル）でのマッチング
   - 再帰的に下位 trie に入っていく
   - 各レベルでキャッシュチェック → `link_flags.get()` → `get_link_simple()` → `prefix_match()`
   - クエリ終端に到達すると `restore_` を呼ぶ

3. **`restore_` (3.3%)** — キー復元（結果バッファにキーのバイト列を構築）
   - ノード ID から根方向に遡りながらバイトを収集
   - `select1` が毎回呼ばれる（親ノード特定のため）
   - predictive_search では **全 hit に対して** 呼ばれる

### BitVector 操作 (~7%)

| 関数 | self % | 用途 |
|---|---|---|
| `select1` | 3.0% | 親ノード特定 (restore_, find_child) |
| `select0` | 0.8% | 子ノード先頭位置の特定 |
| `get` | inline | terminal/link フラグチェック |
| `rank1` | 0.4% | terminal フラグの rank (key_id 計算) |

select1/select0 は binary search + popcount で O(log n)。

### 改善余地のある箇所

#### 1. exact match 専用 API の追加 (最大効果)

現在 `predictive_search` は全子孫を列挙する API。Akaza の bigram/skip-bigram は
1 件だけ欲しいのに、列挙用のステート管理（History, key_buf 等）をセットアップしている。

**提案**: `lookup(key) -> Option<(key_id, key_bytes)>` のような exact match 専用 API。
- History 管理が不要（バックトラック不要）
- `restore_` が不要（結果キーは入力キーのスーパーセット = 入力 6B + 末尾 2B）
- State の Box アロケーションも不要にできる可能性

#### 2. Agent の再利用 / State アロケーション削減

毎回 `Agent::new()` → `state: None` → 初回 predictive_search で `Box<State>` を作成 → drop。
- `State` は内部に `Vec<History>`, `Vec<u8>` (key_buf) を持つ
- 再利用すれば alloc/free を削減可能
- rsmarisa 側で `set_query_bytes` 時に state を reset する機構は既にある
- ただし Akaza 側で exact match API があれば Agent 自体が不要

#### 3. select1/select0 の高速化

現在の実装は binary search + popcount。
- broadword / pdep/pext ベースの O(1) select が知られている
- `select0` は `louds.select0(node_id)` で毎回呼ばれるホットパス
- ただし改善幅は限定的（現状で ~4%）

#### 4. キャッシュヒット率の改善

`predictive_find_child` はまずキャッシュをチェックする:
```rust
let cache_id = (node_id ^ (node_id << 5) ^ (label as usize)) & cache_mask;
if node_id == self.cache[cache_id].parent() { ... }
```
キャッシュミス時は `select0` + 線形探索。ビルド時の `cache_size` パラメータで改善可能。

#### 5. 値の格納方法の見直し（Akaza 側）

現在スコアをキーの末尾に埋め込んでいるため、検索後に `restore_` でキー全体を復元し
末尾 2B を取り出す必要がある。

**提案**: スコアを trie の外に持つ（key_id → score の flat array）。
- `predictive_search` で key_id が取れた時点でスコアが引ける
- `restore_` が不要になる（結果キーの内容を見る必要がない）
- trie のキーが 6B (id1+id2 のみ) になり、trie 自体も小さくなる

## Microbenchmark: rsmarisa vs fst vs sorted-array

18M エントリの bigram.model で比較。ベンチコード: `libakaza/benches/bigram_lookup.rs`

| 形式 | hit (ns/lookup) | miss (ns/lookup) | mixed (ns/lookup) | サイズ |
|---|---|---|---|---|
| rsmarisa (現状) | 953 | 593 | 866 | 69 MB |
| fst | 429 (2.2x↑) | 121 (4.9x↑) | 300 (2.9x↑) | 150 MB |
| sorted-array | 497 (1.9x↑) | 335 (1.8x↑) | 481 (1.8x↑) | 147 MB |

### キー設計バリエーション (fst サイズ比較)

| キー設計 | fst サイズ |
|---|---|
| 3B+3B LE (現状のまま) | 149.8 MB |
| 3B+3B **BE** (prefix 共有改善) | 137.5 MB |
| 21bit+21bit packed BE | 140.7 MB |
| sorted-array (理論下限) | 146.7 MB |

6B バイナリキーは共通 prefix が少なく、fst/sorted-array のサイズは理論下限 (entries × 8B = 147MB) 付近。
rsmarisa が 69MB なのは LOUDS の再帰 patricia trie による圧縮が実際に効いているため。

### 補足データ

- Unigram vocab: 1,058,888 語
- word_id 範囲: 0〜1,058,889 (21bit で収まる、現状は 24bit = 3B)
- Bigram エントリ数: 18,333,516

## 改善方針の優先順位

### rsmarisa 側の改善 (最優先)

1. **exact match / lookup 専用 API** — predictive_search のオーバーヘッドを回避。
   restore_ 不要、History 不要、State アロケーション不要にできる
2. **値をキーから分離** — key_id → score の外部配列。trie サイズ削減 + restore_ 不要
3. **Agent 再利用対応の改善** — reset が軽量になるよう State を再利用可能に
4. **select の高速化** — broadword/pdep ベースの O(1) select

### Akaza 側の改善

1. exact match API を使うように bigram/skip-bigram LM を書き換え
2. 値の外部配列化に対応
3. Agent 再利用（rsmarisa 側で API が整うまでの暫定）

### 未調査事項

- exact match API 実装後の実測効果
- 値の外部配列化による trie サイズ変化の実測
- select 高速化の実効果（現状 ~4% なので ROI は低い）
- エントリ数削減（低頻度 bigram の枝刈り）の影響
