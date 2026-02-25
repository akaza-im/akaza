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
| Bigram LM | 69 MB | `[3B id1][3B id2]` (固定 6B) + 外部 scores | lookup (exact match) |
| Skip-bigram LM | 114 MB | `[3B id1][3B id2]` (固定 6B) + 外部 scores | lookup (exact match) |
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

#### 1. exact match (lookup) API の利用 → **実施済み (PR #520)**

rsmarisa には元々 `lookup` (exact match) API が存在していたが、スコアがキーに
埋め込まれていたため `predictive_search` を使わざるを得なかった。
key/value 分離により `lookup` に切り替え済み。

#### 2. Agent の再利用 → **実施済み**

`RefCell<Agent>` として構造体に保持し、毎回のアロケーションを回避。

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

#### 5. 値の格納方法の見直し（Akaza 側） → **実施済み (PR #520)**

スコアを trie の外に持つ（key_id → score の flat array）方式に変更。
trie のキーが 6B (id1+id2 のみ) になり、`lookup` で key_id を取得後
`scores[key_id]` でスコアを引く。

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

## 実施済みの改善

### 1. Agent 再利用

`Agent::new()` を毎回呼ぶのではなく、`RefCell<Agent>` として構造体に保持し再利用。
`set_query_bytes` が内部で state を reset するため、安全に再利用可能。

### 2. bigram/skip-bigram の key/value 分離 + lookup 切り替え (PR #520)

trie キーからスコア (f16) を除去し、スコアは `key_id` をインデックスにした flat array
（別ファイル `.scores`）で保持するように変更。これにより、rsmarisa に元々存在していた
`lookup` (exact match) API が使えるようになった。

```
旧: trie key = [3B id1][3B id2][2B f16_score]  → predictive_search
新: trie key = [3B id1][3B id2]                 → lookup → scores[key_id]
```

- trie キーが 8B → 6B に縮小（trie サイズも削減）
- `predictive_search` → `lookup` に切り替え（restore_ 不要、History 不要）
- scores ファイルフォーマット: `[u32 LE num_entries][f16 LE × N]`

## 残りの改善候補

1. **select の高速化** — broadword/pdep ベースの O(1) select（現状 ~4% なので ROI は低い）
2. **エントリ数削減** — 低頻度 bigram の枝刈りの影響調査

### 未調査事項

- key/value 分離 + lookup 切り替えの実測効果（ベンチマーク未完了）
- 値の外部配列化による trie サイズ変化の実測
- select 高速化の実効果
- エントリ数削減（低頻度 bigram の枝刈り）の影響
