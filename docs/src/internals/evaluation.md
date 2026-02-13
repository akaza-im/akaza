# 評価方法

このページでは、Akaza のかな漢字変換精度の評価方法を説明する。

## 評価コーパス

評価には [anthy-unicode](https://github.com/fujiwarat/anthy-unicode) プロジェクトに含まれるコーパスを利用している。このコーパスは public domain で公開されており、[calctrans ディレクトリ](https://github.com/fujiwarat/anthy-unicode/tree/main/calctrans)から取得できる。

anthy-unicode プロジェクトおよびコーパスの作成者に感謝する。このコーパスのおかげで、Akaza の変換精度を定量的に評価し、改善の効果を客観的に測定できている。

### コーパスのフォーマット

各行は `|` で文節を区切った読みと正解表記のペアで構成される:

```
|読み1|読み2|...|読みN| |正解1|正解2|...|正解N|
```

例:

```
|uim-fepの|あたらしい|ばーじょん| |uim-fepの|新しい|バージョン|
```

`#` で始まる行はコメント、空行は無視される。

### 使用するファイル

| ファイル | 行数（データ行） | 内容 |
|---|---|---|
| `corpus.0.txt` | 0 | デバッグ用（データなし） |
| `corpus.1.txt` | 1,743 | 一般的な変換テスト |
| `corpus.2.txt` | 23 | 追加テスト |
| `corpus.3.txt` | 9,252 | 大規模テスト |
| `corpus.4.txt` | 10 | 誤変換の例示（**評価には使用しない**） |
| `corpus.5.txt` | 47 | 追加テスト |

`corpus.4.txt` は `~` や `!` で誤変換パターンを記録したファイルであり、正解データではないため評価から除外している。

評価に使用するのは `corpus.0, 1, 2, 3, 5` の 5 ファイルで、合計約 **11,065 件**のテストケースとなる。

## 評価指標

### 参考文献

評価方法は以下の論文に記載の手法を採用している:

> 日本語かな漢字変換における識別モデルの適用とその考察
> [ANLP 2011 C4-6](https://www.anlp.jp/proceedings/annual_meeting/2011/pdf_dir/C4-6.pdf)

### 指標の定義

| 指標 | 説明 |
|---|---|
| **Good** | Top-1（最上位候補）がコーパスの正解と完全一致した件数 |
| **Top-k** | Top-1 では不正解だが、上位 k 個の候補のいずれかに正解が含まれる件数 |
| **Bad** | 上位 k 個のどの候補にも正解が含まれない件数 |
| **再現率** | LCS（最長共通部分列）ベースの文字レベル再現率 |

### 再現率の計算

再現率は、変換結果とコーパスの正解の間の**最長共通部分列（LCS）**を用いて計算する:

```
再現率 = 100 × Σ LCS(正解, 変換結果) / Σ |変換結果|
```

- `LCS(正解, 変換結果)`: 正解文字列と変換結果の最長共通部分列の文字数
- `|変換結果|`: 変換結果の文字数
- 全テストケースについて分子・分母をそれぞれ合計してから割合を求める

完全一致であれば再現率 100% となる。部分的な一致（例: 一部の文節だけ正解）も定量的に評価できる点が利点である。

### 前処理

評価時に以下の前処理を行う:

- **文節区切りの除去**: `|` を除去して平文に変換
- **全角数字の正規化**: `０`〜`９` を半角 `0`〜`9` に変換

## 評価の実行

`akaza-data evaluate` コマンドで評価を実行する。[akaza-default-model](https://github.com/akaza-im/akaza-default-model) リポジトリ内で以下のように使用する:

```bash
akaza-data evaluate \
  --corpus anthy-corpus/corpus.0.txt \
  --corpus anthy-corpus/corpus.1.txt \
  --corpus anthy-corpus/corpus.2.txt \
  --corpus anthy-corpus/corpus.3.txt \
  --corpus anthy-corpus/corpus.5.txt \
  --eucjp-dict skk-dev-dict/SKK-JISYO.L \
  --utf8-dict data/SKK-JISYO.akaza \
  --model-dir data/ -vv
```

評価はマルチスレッドで並列実行される。

### リランキング重みの評価

リランキング重みを変更して評価する場合は、`--bigram-weight`、`--unknown-bigram-weight`、`--length-weight`、`--skip-bigram-weight` オプションを指定する:

```bash
akaza-data evaluate \
  --corpus ... \
  --length-weight 2.5 \
  ...
```

## 実装

**実装**: `akaza-data/src/subcmd/evaluate.rs`
