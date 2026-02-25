# akaza-corpus-stats

[Akaza](https://github.com/akaza-im/akaza) (Japanese kana-kanji conversion engine) 用の n-gram 統計データを生成するパイプラインです。

日本語 Wikipedia、青空文庫、CC-100 Japanese のテキストをトーカナイズし、unigram/bigram の wordcnt trie と語彙ファイルを生成します。
生成物は [`default-model/`](../default-model/) で `learn-corpus` の入力として使用されます。

## 生成物

| ファイル | 内容 | サイズ目安 |
|---|---|---|
| `dist/stats-vibrato-unigram.wordcnt.trie` | Unigram wordcnt (marisa-trie) | ~28MB |
| `dist/stats-vibrato-bigram.wordcnt.trie` | Bigram wordcnt (marisa-trie) | ~186MB |
| `dist/vibrato-ipadic.vocab` | 語彙リスト (頻度閾値=16) | ~55MB |

## ビルド

### 前提

- `akaza-data` (Rust): `cargo install --git https://github.com/akaza-im/akaza.git akaza-data`
- `wget`, `unzip`, `zstd`
- Python 3 (標準ライブラリのみ使用)
- git submodule の初期化: `git submodule update --init`

### 実行

```bash
make          # フルビルド (初回は Wikipedia ダンプのダウンロードに時間がかかる)
make dist     # dist/ に成果物を出力
```

## データソース

### Japanese Wikipedia (CirrusSearch dump)

- URL: `https://dumps.wikimedia.org/other/cirrussearch/`
- 形式: gzip 圧縮 NDJSON (テンプレート展開済みプレーンテキスト)
- `scripts/extract-cirrus.py` でストリーミング展開

### 青空文庫

- git submodule `aozorabunko_text` で取得
- 著作権の消滅した日本語文学作品のテキストアーカイブ

### CC-100 Japanese

- URL: `https://data.statmt.org/cc-100/ja.txt.xz`
- 形式: xz 圧縮プレーンテキスト (1 行 1 文、空行で文書区切り)
- `scripts/extract-cc100.py` で `<doc>` 形式に変換
- `CC100_LIMIT` 変数で処理文書数を制限可能 (デフォルト 0 = 無制限)

## 数字トークンの `<NUM>` 正規化

`akaza-data` の unigram/bigram 生成時に、数字プレフィックスを持つトークンを `<NUM>` に正規化してカウントを集約します。

| 正規化前 | 正規化後 |
|---|---|
| `1匹/1ひき`, `2匹/2ひき`, `100匹/100ひき` | `<NUM>匹/<NUM>ひき` |
| `1年/1ねん`, `2019年/2019ねん` | `<NUM>年/<NUM>ねん` |
| `1/1`, `2019/2019` | `1/1`, `2019/2019` (変換なし) |

これにより、コーパス中で低頻度な「2匹」「3匹」等も、「1匹」等と集約された高頻度の LM スコアを共有でき、数字+助数詞パターン全般の変換精度が向上します。

正規化は surface が「ASCII 数字 + 非数字の接尾辞」のトークンにのみ適用されます。裸の数字（`1/1` 等）は正規化しません（全数字カウントが集約されると `<NUM>/<NUM>` のスコアが極端に高くなり、助詞「に」「さん」等が数字に化ける退行を防ぐため）。`第1回` のように非数字で始まるものも対象外です。

## ライセンス

### スクリプト・設定ファイル

このリポジトリ内のスクリプトおよび設定ファイル (`scripts/`, `Makefile`, `mecab-user-dict.csv` 等) は MIT License で提供されます。詳細は [LICENSE](LICENSE) を参照してください。

### 生成データ

生成される統計データ (wordcnt trie, vocab) は以下のデータソースに由来する派生物です。

- **Japanese Wikipedia**: [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) (Wikimedia Foundation)
- **青空文庫**: パブリックドメイン (著作権の消滅した作品)
- **CC-100**: 元データは Common Crawl から抽出 ([CC-100 paper](https://aclanthology.org/2020.lrec-1.494/))

Wikipedia 由来のデータを含むため、生成物の再配布には CC BY-SA 4.0 の条件が適用されます。

### 使用する外部ツール・辞書

- **Vibrato** (MeCab 互換トーカナイザー): 辞書データは Apache-2.0 / BSD ライセンス (IPADIC)
- **akaza-data**: MIT License

## リリース

データ量が大きいため CI ではビルドせず、ローカルでビルドして GitHub Release にアップロードする運用です。

### 更新手順

```bash
# 1. submodule を最新に更新
git submodule update --init

# 2. フルビルド (初回は Wikipedia ダンプのダウンロードに時間がかかる)
make dist

# 3. CalVer タグを自動生成し、GitHub Release を作成
#    gh CLI (https://cli.github.com/) が必要
make release
```

`make release` は以下を自動で行います:
1. `dist/` に成果物を生成 (未ビルドなら自動でビルド)
2. CalVer タグ (`vYYYY.MMDD.PATCH`) を生成・push
3. tarball を作成して GitHub Release にアップロード

同日に複数回実行すると PATCH が自動インクリメントされます (`v2026.0207.0` → `v2026.0207.1` → ...)。
