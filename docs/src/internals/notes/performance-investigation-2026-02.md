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

---

# rsmarisa チューニング (2026-06-01)

`akaza-data bench`（`anthy-corpus/corpus.3.txt`, 50文, k=5）と perf -F 997 で計測。
ローカル rsmarisa を `[patch.crates-io] rsmarisa = { path = "../rsmarisa" }` で参照。

## ベンチ結果（best of 3）

| 指標 | baseline (rsmarisa 0.4.0) | 最適化後 |
|---|---|---|
| avg | 5.4–5.5 ms | 5.2–5.3 ms |
| median | 3.5 ms | 3.3–3.5 ms |
| p95 | 18.2–18.5 ms | 17.2–17.8 ms |
| p99 | 30.1–30.4 ms | 29.2–29.7 ms |
| max | 40.3–40.7 ms | 38.4–40.9 ms |

おおむね 2–5% の改善。

## perf self-time 内訳（cpu_core, 300文）

| 関数 | baseline | 最適化後 | 差 |
|---|---|---|---|
| `LoudsTrie::find_child` | 16.37% | 14.53% | -1.84pp |
| `LoudsTrie::match_` | 6.10% | 5.98% | -0.12pp |
| `LoudsTrie::prefix_match_` | 2.52% | 2.31% | -0.21pp |
| `BitVector::select0` | 2.67% | 2.11% | -0.56pp |
| `LoudsTrie::predictive_search` | 1.97% | 1.90% | -0.07pp |
| `LoudsTrie::restore_` | 1.63% | 1.69% | +0.06pp |
| `BitVector::select1` | 1.38% | 1.02% | -0.36pp |
| `BitVector::rank1` | 0.79% | 0.75% | -0.04pp |
| **rsmarisa 合計** | **~33.4%** | **~30.3%** | **-3.1pp** |

select0/select1 は PDEP 化が効いて約 20–26% 相対減。

## 実施した最適化（rsmarisa 側）

1. **PDEP ベースの `select_bit_u64`** — x86_64 で BMI2 を初回 `is_x86_feature_detected!`
   で検出して以降キャッシュ。8 バイトのテーブルルックアップループ →
   `_pdep_u64(1 << i, unit).trailing_zeros()` の 2-3 命令へ。
2. **`assert!` → `debug_assert!`** — `BitVector::{get, rank0, rank1}`,
   `State::{set_node_id, set_query_pos, set_history_pos}`, `LoudsTrie::{find_child,
   predictive_find_child, match_, prefix_match_, restore_}` の冒頭境界チェック。
   release ビルドの分岐数削減。
3. **`Tail::match_tail` / `prefix_match` の Vec alloc 撤去** — それぞれ毎回
   `agent.query().as_bytes().to_vec()` で 6 バイト程度の `Vec` を確保していた。
   `Agent::query_bytes_and_state_mut()` の split-borrow ヘルパで `&[u8]` と
   `&mut State` を同時に取れるようにし、ループ中の alloc/free を排除。
4. **キャッシュエントリのローカルコピー** — `find_child` / `match_` /
   `prefix_match_` / `restore_` / `predictive_find_child` の hot ループ内で
   `self.cache[cache_id]` を 1 度ローカルにコピー（`Cache` は 12B Copy）。
   `parent/child/extra/link/label` の連続アクセスでの再 indexing を抑える。

いずれも結果は bit-exact（rsmarisa 全 323 ユニットテストが debug ビルドで pass、
release ではプレ存在の `should_panic` テスト 3 件のみ失敗で、これは元から
release で debug_assert が無効になることに依存していた既存のテストバグ）。

## 残りのボトルネック

- `find_child` の 14.5% は LOUDS の cache 照合 + 子ノード線形探索が支配的。
  キャッシュサイズ（trie 構築時の `cache_level`）を上げると改善余地あり。
- `resolve_k_best` 15.7%, quicksort 17%, sip::Hasher 2.2% は akaza 側 DP の
  コスト。FxHashMap のはずなのに sip::Hasher が出ているのは要調査。
- libc malloc/memmove で計 8% 前後。`resolve_k_best` の `Vec<KBestEntry>` 再確保
  などが寄与している可能性。

---

# 起動時間短縮 (2026-06-01)

`akaza-data bench --dict-cache` の `Engine built in Xms` 行で計測。

## 起動時間の内訳

| 段階 | 元 (warm) | cold | warm |
|---|---|---|---|
| unigram.model load | 7ms | 11ms | 7ms |
| bigram.model + scores load | 154ms | 59〜64ms | 32ms |
| skip_bigram.model + scores load | 254ms | 94〜104ms | 56ms |
| dict load (cache hit) | 7ms | 12〜20ms | 6ms |
| single_term | <1ms | <1ms | <1ms |
| **kana_trie 構築** | **1069ms** | **3〜4ms** | **1.5〜1.7ms** |
| **合計 (Engine built)** | **1490ms** | **188〜195ms** | **102〜104ms** |

cold は `posix_fadvise(POSIX_FADV_DONTNEED)` で当該ファイルを page cache から
追い出した状態。OS read のみで占められる下限値。

## 実施した最適化

### 1. `MarisaKanaTrie` の導入と cache 化（最大の効果）

`CedarwoodKanaTrie` を毎起動 `dict.yomis()` (約 1M 件) + `single_term.yomis()`
で `update()` していたのが起動時間の 70% 強を占めていた。同じ集合は変わらない
データなので marisa-trie に build し `~/.cache/akaza/kana_trie_cache.marisa`
として永続化、次回以降は `Trie::load` (約 2-4ms) で済むようにした。

- 鮮度判定: `kana_trie_cache.marisa` の mtime が `kana_kanji_cache.marisa`
  より古ければ再構築。dict 側 cache と同期する。
- `dict_cache=false` のときは従来の cedarwood 経路にフォールバックし
  cache ファイルを汚さない。
- `KanaTrie` trait 実装で `Segmenter` への接続は変更不要。

### 2. `.scores` ファイルの一括 read

bigram (18M entry, 35MB) / skip_bigram (30M entry, 58MB) の `.scores` を
2 バイトずつ `read_exact` + `push` する loop で読んでいた。`Vec<f16>` を
1 回確保し、その underlying byte slice に対して 1 回の `read_exact` で埋める形
に変更。`f16` (`#[repr(transparent)] u16`) と LE x86_64 の組み合わせで
そのまま reinterpret 可能。

### 3. bench へ `--dict-cache` フラグを追加

bench は元々 `dict_cache=false` で動作していた（ユーザーの cache を汚さない）。
起動時間を計測したいときだけ opt-in で有効化できるようにした。

## 設計上のトレードオフ

- **真の zero-copy mmap は採用しなかった**。rsmarisa の現在の `Trie::mmap()` は
  mmap した領域から所有 Vec へ `copy_nonoverlapping` するので、純粋な
  speed-up は load 経路をやや短縮する程度。一方 zero-copy 化には rsmarisa の
  `Vector<T>` を「所有 / mmap 借用」の enum に作り変える必要があり、akaza 側で
  目標 (500ms 以下) を満たすには A・B で十分だったため見送り。
- **scores の `unsafe` は LE プラットフォーム前提**。BE arch をサポートする
  必要が出たら portable な `chunks_exact + from_le_bytes` 経路に切り替える。

## 残りの最適化余地

- skip_bigram model+scores の load を std::thread で並列化すると 150-200ms 削れる
  はず（今 cold で 100ms 強なので、`Engine built` を 100ms 切る目処）。
- skip_bigram_weight=0 のときに skip_bigram の load 自体をスキップする
  (lazy load)。convert 側で重み 0 のフォールバックは既に skip-bigram cost を
  0 で扱う実装になっているため、`skip_bigram_lm: None` で動かせるはず。
