# akaza-data

辞書および言語モデルの管理用ツールです。

## インストール

```bash
cargo build --package akaza-data --release
```

## コマンド一覧

### check - かな漢字変換を実行する

ひらがな文字列をかな漢字変換して結果を表示します。
引数を省略すると stdin から行ごとに読み取ります（エンジンを1回だけ構築して複数行を処理できます）。

```bash
# 基本的な使い方（設定ファイルのデフォルト設定を使用）
akaza-data check きょうはいいてんきですね
# => 今日/は/いい/天気/です/ね

# stdin から複数行を処理
echo -e "きょうはいいてんきですね\nわたしはにほんごがすきです" | akaza-data check

# JSON 出力
akaza-data check --format json きょうはいいてんきですね

# 複数候補を表示
akaza-data check --format json --candidates 3 きょうはいいてんきですね

# ユーザー学習データを使用
akaza-data check --user-data きょうはいいてんきですね

# モデルディレクトリを指定
akaza-data check --model-dir /path/to/model きょうはいいてんきですね

# k-best: 上位 k 個の分節パターン（文節の区切り方）を表示
akaza-data check --k-best 5 きたかなざわ
# => [1] 北/金沢 (cost: ...)
# => [2] 来た/かなざわ (cost: ...)
# => ...

# k-best + JSON
akaza-data check --k-best 3 --format json きょうはいいてんきですね
```

### evaluate - 変換精度を評価する

コーパスを使って変換精度を評価します。

```bash
akaza-data evaluate --corpus corpus.txt --model-dir /path/to/model
```

### tokenize - コーパスをトーカナイズする

コーパスを形態素解析器でトーカナイズします。

```bash
akaza-data tokenize --reader jawiki --system-dict /path/to/unidic src_dir dst_dir
```

`--reader` には `jawiki`, `aozora_bunko` が指定可能です。

### tokenize-line - 単一文をコーパス形式に変換する

あらかじめ読みたい文を CLI 引数として渡し、`surface/yomi` を 1 行で出力します。たとえば:

```bash
akaza-data tokenize-line --system-dict /path/to/dict/system.dic --kana-preferred 「わたしはにほんごがすきです。」
```

`--user-dict` でユーザー辞書を追加することもでき、出力先は標準出力です。

### wfreq - 単語頻度ファイルを生成する

トーカナイズされたコーパスから単語頻度ファイルを生成します。

```bash
akaza-data wfreq --src-dir tokenized_corpus output.wfreq
```

### vocab - 語彙リストを生成する

単語頻度ファイルから語彙リストを生成します。

```bash
akaza-data vocab --threshold 10 input.wfreq output.vocab
```

### make-dict - システム辞書を作成する

語彙リストとコーパスからシステム辞書を作成します。

```bash
akaza-data make-dict --corpus corpus_dir --unidic /path/to/unidic --vocab vocab.txt output.txt
```

### wordcnt-unigram - ユニグラム言語モデルを作成する

単語頻度ファイルからユニグラム言語モデルを作成します。

```bash
akaza-data wordcnt-unigram input.wfreq output.model
```

### wordcnt-bigram - バイグラム言語モデルを生成する

コーパスからバイグラム言語モデルを生成します。

```bash
akaza-data wordcnt-bigram --threshold 5 --corpus-dirs corpus_dir unigram.model bigram.model
```

### learn-corpus - 言語モデルを学習する

コーパスから言語モデルのパラメータを学習します。変換結果が正解と一致しない場合に、正解側のユニグラム・バイグラムのスコアを調整（delta 分だけ加算）することで学習を行います。

```bash
akaza-data learn-corpus --delta 1 \
    --may-epochs 10 --should-epochs 100 --must-epochs 1000 \
    corpus/may.txt corpus/should.txt corpus/must.txt \
    src_unigram.model src_bigram.model dst_unigram.model dst_bigram.model
```

#### コーパスファイル（may.txt / should.txt / must.txt）

3 つのコーパスファイルは優先度別に分かれており、それぞれ学習時のエポック数（繰り返し回数）が異なります。

| ファイル | デフォルトエポック数 | 用途 |
|---|---|---|
| `may.txt` | 10 | 変換できると望ましいが、必須ではないケース |
| `should.txt` | 100 | 正しく変換されるべきケース |
| `must.txt` | 1000 | 必ず正しく変換されなければならないケース |

エポック数が多いほど、そのコーパスの変換結果が正解に近づくまで繰り返し学習します（全文正解になった時点で早期終了します）。

#### コーパスファイルの形式

[Kytea のフルアノテーションコーパス](http://www.phontron.com/kytea/io-ja.html)と同様の形式で、`表層形/読み` をスペース区切りで記述します。

```
今日/きょう は/は いい/いい 天気/てんき です/です ね/ね
私/わたし の/の 名前/なまえ は/は 中野/なかの です/です
```

- `;;` で始まる行はコメントとして無視されます
- 空行は無視されます
- 表層形と読みが同じ場合（ひらがなのみの語など）は `は/は` のように同じ文字列を書きます

### dump-unigram-dict / dump-bigram-dict - 辞書をダンプする

言語モデルの内容をテキスト形式でダンプします。

```bash
akaza-data dump-unigram-dict unigram.model
akaza-data dump-bigram-dict unigram.model bigram.model
```

## 詳細なヘルプ

各コマンドの詳細なオプションは `--help` で確認できます。

```bash
akaza-data check --help
akaza-data evaluate --help
```
