# Akaza (ibus-akaza)

Yet another kana-kanji-converter on IBus, written in Rust.

統計的かな漢字変換による日本語IMEです。
Rust で書いています。

**現在、開発途中のプロダクトです。非互換の変更が予告なくはいります**

[![CI](https://github.com/akaza-im/akaza/actions/workflows/ci-simple.yml/badge.svg)](https://github.com/akaza-im/akaza/actions/workflows/ci-simple.yml)

## 関連プロジェクト

* [akaza-default-model](default-model/) - デフォルト言語モデル
* [jawiki-kana-kanji-dict](https://github.com/tokuhirom/jawiki-kana-kanji-dict) - Wikipedia ベース SKK 辞書

## モチベーション

いじりやすくて **ある程度** UIが使いやすいかな漢字変換があったら面白いなと思ったので作ってみています。
「いじりやすくて」というのはつまり、Hack-able であるという意味です。

モデルデータを自分で生成できて、特定の企業に依存しない自由なかな漢字変換エンジンを作りたい。

## 特徴

* **Rust で実装**: UI/Logic をすべて Rust で書いてあるので、拡張が容易です
* **統計的かな漢字変換**: 2gram 言語モデルを採用
    * 言語モデルの生成元は日本語 Wikipedia と青空文庫です
    * 形態素解析器 [Vibrato](https://github.com/daac-tools/vibrato) で分析した結果をもとに構築
    * 利用者の環境で 1 から言語モデルを再生成することが可能です
* **学習機能**: ユーザー環境で、利用者の変換結果を学習します (unigram, bigram の頻度を学習)
* **GUI 設定ツール**: GTK4 ベースの設定ツールを提供
    * `akaza-conf`: キーマップ、辞書、モデルなどの設定
    * `akaza-dict`: ユーザー辞書の編集
* **SKK 辞書対応**: SKK 形式の辞書ファイルを複数読み込み可能

## かな漢字変換の仕組み

Akaza は **単語 bigram モデルに基づく統計的かな漢字変換** を行っています。
変換は大きく 3 つのフェーズで構成されます。

### 1. セグメンテーション（単語分割）

入力されたかな文字列を、辞書に登録されている単語の読みに基づいて分割候補を列挙します。

- **MARISA Trie** に格納されたシステム辞書と、**Cedarwood Trie** に格納されたユーザー辞書を用いて共通接頭辞検索を行い、可能な単語境界をすべて列挙します
- BFS（幅優先探索）で全分割パターンを探索し、`{終了位置 → [読み候補]}` のマップを生成します
- 数字、未知文字（辞書に存在しない文字）の自動認識も行います

**例**: `"わたしのなまえ"` → `{6: ["わた", "わたし"], 9: ["し", "の"], ...}`

### 2. ラティス（単語グラフ）構築

セグメンテーション結果をもとに、各位置での変換候補を含むラティスグラフを構築します。

- 各読みに対して、システム辞書・ユーザー辞書から漢字候補を検索します
- 辞書に存在しない読みには、ひらがな・カタカナのフォールバック候補を自動生成します
- 数字の漢数字変換、日付・時刻の動的変換なども候補に追加されます
- 各ノードは `(表層形, 読み, 単語ID, ユニグラムコスト)` を保持します

**例**: 位置 0〜9 のノード:
```
[BOS] → [私/わたし, わたし/わたし, ワタシ/わたし, ...] → ... → [EOS]
```

### 3. ビタビアルゴリズムによる最適経路探索

ラティスグラフ上で **ビタビアルゴリズム**（動的計画法）を用いて、最もコストの低い変換候補列を求めます。

**コストの構成要素**:

| コスト | 説明 | 算出方法 |
|--------|------|----------|
| **ユニグラムコスト** | 単語の出現しやすさ | `-log10(P(word))` （加法スムージング適用） |
| **バイグラムコスト** | 単語間の遷移しやすさ | `-log10(P(word_n \| word_{n-1}))` |

- **前向きパス**: BOS（文頭）から EOS（文末）に向かって、各ノードの累積最小コストと最良の前ノードを記録します
- **後ろ向きパス**: EOS から BOS へ最良前ノードをたどり、最適な変換結果を取得します
- ユーザーの学習データがある場合、システムモデルより優先して参照されます

### 4. 動的変換（数字・日付など）

辞書に登録しきれない数字などの変換は、**動的変換マーカー**の仕組みで処理しています。

#### 処理の流れ

1. **Segmenter**: 正規表現 `^(?:0|[1-9][0-9]*)(\.[0-9]*)?` で数字列を検出し、1 トークンとして切り出す
2. **GraphBuilder**: 数字トークンに対して、surface を `"(*(*(NUMBER-KANSUJI"` というマーカー文字列にした WordNode をラティスに追加する
3. **GraphResolver**: 通常通りビタビ探索を行い、Candidate を生成（surface はマーカーのまま）
4. **表示時**: `Candidate::surface_with_dynamic()` が呼ばれ、`int2kanji()` で漢数字に変換される

#### 変換例

| 入力 | 漢数字変換 |
|------|-----------|
| `0` | `零` |
| `10` | `十` |
| `365` | `三百六十五` |
| `10000` | `一万` |

#### 設計のポイント

- **辞書に数字を登録しない**: 無限にある数値パターンを辞書でカバーするのは不可能なため、正規表現で検出して動的に変換する
- **遅延評価**: マーカー文字列をラティスに入れておき、実際の変換は表示時に行う。これにより、ラティス構築・ビタビ探索のロジックを数字のために特殊化する必要がない
- **拡張可能**: `"(*(*(..."` マーカーの仕組みは数字以外（日付・時刻など）にも利用されている

### 辞書とモデルの構成

```
                    ┌─────────────────┐
                    │   入力（かな）   │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌────────────┐  ┌────────────┐  ┌────────────┐
     │システム辞書│  │ユーザー辞書│  │ SKK 辞書   │
     │(MARISA)    │  │(Cedarwood) │  │(複数読込)  │
     └─────┬──────┘  └─────┬──────┘  └─────┬──────┘
           └───────────┬───┘───────────────┘
                       ▼
              ┌────────────────┐
              │  ラティス構築  │
              └───────┬────────┘
                      │
           ┌──────────┼──────────┐
           ▼          ▼          ▼
    ┌───────────┐ ┌────────┐ ┌────────────────┐
    │ unigram   │ │bigram  │ │ユーザー学習    │
    │ .model    │ │.model  │ │(unigram/bigram)│
    │(MARISA)   │ │(MARISA)│ │(.txt)          │
    └─────┬─────┘ └───┬────┘ └───────┬────────┘
          └────────┬───┘──────────────┘
                   ▼
          ┌─────────────────┐
          │ ビタビ最適経路  │
          └────────┬────────┘
                   ▼
          ┌─────────────────┐
          │  変換結果出力   │
          └─────────────────┘
```

**システム辞書** (`SKK-JISYO.akaza`): 日本語 Wikipedia・青空文庫から抽出した読み→漢字の対応表。MARISA Trie で格納。SKK-JISYO.L に含まれない語彙を補完する役割で、SKK-JISYO.L などと併用することを想定しています。

**言語モデル**:
- `unigram.model`: 各単語の出現コスト。MARISA Trie に `(単語ID 24bit, スコア f32)` を格納。
- `bigram.model`: 単語間の遷移コスト。MARISA Trie に `(単語ID1 3byte + 単語ID2 3byte + スコア f16)` を格納。スコアは `-log10(確率)` で表現。
- 学習コーパスは日本語 Wikipedia と青空文庫を [Vibrato](https://github.com/daac-tools/vibrato) で形態素解析して構築。
- 加法スムージング（α=0.00001）でゼロ頻度問題に対応。

**ユーザー学習データ** (`~/.local/share/akaza/`):
- `unigram.v1.txt` / `bigram.v1.txt`: ユーザーが確定した変換結果の頻度統計
- `SKK-JISYO.user`: ユーザー定義の読み→漢字辞書
- 変換確定時に自動更新され、次回変換時にシステムモデルより優先されます

### モデル構築パイプライン

デフォルトモデルは [`default-model/`](default-model/) ディレクトリで構築されます。日本語 Wikipedia・青空文庫・CC-100 をトークン化して n-gram 統計を生成し（[`corpus-stats/`](corpus-stats/)）、手作業コーパスで補正してモデルとシステム辞書を出力します。

詳細なパイプラインは [default-model/README.md](default-model/README.md#how-it-works) を参照してください。

### 数字トークンの `<NUM>` 正規化

言語モデルの構築時およびランタイムの LM lookup 時に、数字+接尾辞のトークン（例: `1匹`, `2019年`）を `<NUM>` に正規化してカウントを集約し、数字+助数詞パターン全般の変換精度を向上させています。

正規化ルールの詳細は [corpus-stats/README.md](corpus-stats/README.md#数字トークンの-num-正規化) を参照してください。

### チューニングポイント

誤変換の原因に応じて、適切な調整方法があります。

- **`corpus/*.txt`**（bigram スコア調整）: 同音異義語の文脈判別、分節解析の補強、誤候補の抑制。コーパスに書かれた単語はシステム辞書にも自動登録されます。
- **`mecab-user-dict.csv`**（Vibrato トークン化の修正）: Vibrato が未知語として扱う単語や分割を誤る複合語を登録。訓練時のトークン化に影響し、IME の分節解析には間接的に影響します。
- **`dict/SKK-JISYO.akaza`**（システム辞書の補完）: SKK-JISYO.L に含まれない新語・固有名詞・専門用語を追加。

詳細な調整ガイドは [default-model/README.md](default-model/README.md#調整方法) を参照してください。

### 現状の制約

#### bigram モデルの原理的な限界

Akaza は単語 bigram モデルを採用しているため、**直前の 1 単語しか文脈として参照できない** という原理的な制約があります。

**bigram で対応できる例**:
- 「板が**厚い**」vs「お湯が**熱い**」 — 直前の名詞で同音異義語を区別可能

**bigram では対応が困難な例**:
- 「夏は**暑い**」vs「この板は**厚い**」 — 直前の助詞「は」が同じため区別不能。共起情報（離れた単語との関連性）が必要
- 文末の意志形「だらだら**しよう**」→「使用」に誤変換 — 直前の語がさまざまで bigram では汎用的に抑制困難
- 「猫を**飼う**」→「買う」に誤変換 — 直前の助詞「を」が同じため区別不能

#### BOS/EOS bigram

BOS（文頭）と EOS（文末）に `word_id` を付与し、bigram モデルに含めています。これにより、文頭に来やすい単語（「私は」「今日は」など）や文末に来やすい単語の情報が変換精度の向上に活用されます。古いモデルファイル（BOS/EOS エントリなし）では従来通りデフォルトコストにフォールバックします。

#### Skip-bigram（1語飛ばし bigram）

隣接 bigram に加えて、1語飛ばした単語ペア (w_{i-2}, w_i) の共起頻度を Viterbi DP に統合しています。これにより、「夏は**暑い**」と「板は**厚い**」のように助詞を挟んだ共起関係を捉え、bigram だけでは判別できない同音異義語の解消を補助します。

#### k-best の実装

ビタビアルゴリズムを拡張した k-best Viterbi を実装しています。各ノードに対して上位 k 個の `(cost, prev_node, prev_rank)` エントリを保持し、EOS から逆方向にたどることで**分節の区切り方が異なるパス**を k 本列挙します。Tab キーでパスを切り替え可能です。

現在の実装では、各ノードの上位 k エントリは `Vec` をソートして `truncate(k)` する方式です。

#### 学習データの制約

OSS として持続可能な開発を行うため、学習データは Wikipedia と青空文庫に限定しています。無料で利用可能な GitHub Actions のリソースで処理できる分量に収めているため、大規模な商用かな漢字変換エンジンと比べるとデータ量に限りがあります。口語表現やビジネス表現のカバーが薄いという弱点があります。

### 今後の改善案

以下は、上記の制約を踏まえて検討している改善案です。

#### Trigram 以上の高次 n-gram モデル

現在の bigram（2単語間）から trigram（3単語間）に拡張することで、より広い文脈を考慮した変換が可能になる。

- **期待される効果**: 「AはB」のように助詞を挟むパターンで、2 単語前の名詞を参照できる。
- **課題**: モデルサイズの増大。bigram でも MARISA Trie で圧縮しているが、trigram ではエントリ数が大幅に増加する。スパースデータ問題も深刻化する。

#### ニューラル言語モデルの導入

Transformer ベースの軽量モデルを用いたリスコアリングにより、統計モデルでは捉えきれない長距離の文脈依存性を扱う。

- **期待される効果**: 文全体の意味を考慮した変換が可能になり、bigram の原理的限界を根本的に解消できる。
- **課題**: 推論速度（IME はリアルタイム応答が必要）。モデルサイズ（オンデバイスで動作する必要がある）。n-best のリスコアリングに限定すれば負荷を抑えられる可能性がある。

#### 文節区切りの最適化

「一日中」（いちにちじゅう）が「一日+十」に分節されるなど、分節解析が正しい複合語を分割してしまうケースへの対応。

#### 予測変換（サジェスト）

入力途中での変換候補の提示。入力効率の向上が期待できる。

#### 学習コーパスの拡充

LLM（ChatGPT、Claude 等）による日本語文の生成を活用し、口語表現・ビジネス表現・季節の挨拶など Wikipedia に出にくい表現のカバーを強化する。LLM 生成文は著作権の問題がなく Public Domain として扱える。

## 用語集（Glossary）

| 用語 | 説明 |
|------|------|
| **PreComposition** | 未入力状態。preedit が空で、IME がキー入力を待っている状態 |
| **Composition** | preedit にローマ字/ひらがなが入力されているが、まだ変換（Space）を押していない状態。サジェスト有効時は候補ポップアップが表示されることがある |
| **Conversion** | Space を押して変換候補が確定候補として表示されている状態。文節の選択や候補の切り替えが可能 |
| **preedit** | 入力中の未確定文字列。アプリケーションのカーソル位置にインラインで表示される |
| **lookup table** | 変換候補のポップアップウィンドウ。IBus が提供する候補一覧 UI |
| **clause（文節）** | 変換結果を構成する単位。例: 「今日は/いい/天気です」の各区切り |
| **k-best** | ビタビアルゴリズムで上位 k 個の分節パターンを列挙する手法。Tab キーで切り替え可能 |
| **サジェスト** | Composition 中にひらがな2文字以上入力された時点で、変換候補をポップアップ表示する機能。preedit はひらがなのまま |
| **ライブ変換** | Composition 中にリアルタイムで変換結果を preedit に反映する機能。サジェストとは異なり preedit 自体が漢字に変わる |
| **auxiliary text** | lookup table と併せて表示される補助テキスト。選択中の文節の読みなどを表示 |
| **commit（確定）** | 変換結果をアプリケーションに送信し、入力を完了すること |

## Dependencies

### Runtime dependencies

* ibus 1.5+
* marisa-trie (libmarisa)
* GTK 4.0+ (設定ツール用)

### Build time dependencies

* Rust 1.92.0+ (stable toolchain)
* Cargo
* pkg-config
* clang
* libibus-1.0-dev
* libmarisa-dev
* libgtk-4-dev
* libgirepository1.0-dev

### Supported environment

* **OS**: Linux 6.0 以上
* **IBus**: 1.5 以上

## Install 方法

### 1. ビルド依存関係のインストール

Ubuntu/Debian の場合:

```bash
sudo apt-get update
sudo apt-get install ibus libgirepository1.0-dev libmarisa-dev clang libibus-1.0-dev libgtk-4-dev pkg-config
```

### 2. Rust のインストール

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install stable
```

### 3. ibus-akaza のビルドとインストール

```bash
git clone https://github.com/akaza-im/akaza.git
cd akaza
make
sudo make install
```

`make install` により、モデルファイル（[`default-model/`](default-model/)）のダウンロードとインストールも自動で行われます。

### 4. IBus の再起動と有効化

```bash
ibus restart
ibus engine akaza
```

※ 再ログインは不要です。必要な場合は `ibus restart` で再起動できます。

または、IBus の設定画面から Akaza を追加してください。

## 設定方法

### GUI 設定ツール（推奨）

インストール後、以下のコマンドで GUI 設定ツールを起動できます：

```bash
# 一般設定（キーマップ、辞書、モデルなど）
akaza-conf

# ユーザー辞書の編集
akaza-dict
```

GUI ツールを使用すると、キーマップの選択、SKK 辞書の追加、モデルの切り替えなどが簡単に行えます。

### 手動設定（上級者向け）

設定ファイルは `~/.config/akaza/config.yml` に保存されます。

#### Keymap の設定

Akaza は典型的には以下の順番で探します。

1. `~/.local/share/akaza/keymap/{KEYMAP_NAME}.yml`
2. `/usr/local/share/akaza/keymap/{KEYMAP_NAME}.yml`
3. `/usr/share/akaza/keymap/{KEYMAP_NAME}.yml`

このパスは、[XDG ユーザーディレクトリ](https://wiki.archlinux.jp/index.php/XDG_%E3%83%A6%E3%83%BC%E3%82%B6%E3%83%BC%E3%83%87%E3%82%A3%E3%83%AC%E3%82%AF%E3%83%88%E3%83%AA)
の仕様に基づいています。
Akaza は Keymap を `XDG_DATA_HOME` と `XDG_DATA_DIRS` から探します。
`XDG_DATA_HOME` は設定していなければ `~/.local/share/` です。`XDG_DATA_DIRS` は設定していなければ `/usr/local/share:/usr/share/` です。

#### RomKan の設定

ローマ字かなマップも同様のパスから探します。

1. `~/.local/share/akaza/romkan/{ROMKAN_NAME}.yml`
2. `/usr/local/share/akaza/romkan/{ROMKAN_NAME}.yml`
3. `/usr/share/akaza/romkan/{ROMKAN_NAME}.yml`

設定変更は `akaza-conf` の GUI で行うことを推奨します。

#### Model の設定

Model は複数のファイルからなります：

- `unigram.model`
- `bigram.model`
- `SKK-JISYO.akaza`

以下のパスから読み込まれます：

- `~/.local/share/akaza/model/{MODEL_NAME}/unigram.model`
- `~/.local/share/akaza/model/{MODEL_NAME}/bigram.model`
- `~/.local/share/akaza/model/{MODEL_NAME}/SKK-JISYO.akaza`

keymap, romkan と同様に、`XDG_DATA_DIRS` からも読むことができます。

## FAQ

### 最近の言葉が変換できません/固有名詞が変換できません

流行り言葉が入力できない場合、[jawiki-kana-kanji-dict](https://github.com/tokuhirom/jawiki-kana-kanji-dict) の利用を検討してください。
Wikipedia から自動的に抽出されたデータを元に SKK 辞書を作成しています。
Github Actions で自動的に実行されているため、常に新鮮です。

一方で、自動抽出しているために変なワードも入っています。変なワードが登録されていることに気づいたら、github issues で報告してください。

### 人名が入力できません

必要な SKK 辞書を読み込んでください。

**GUI での設定（推奨）**:
1. `akaza-conf` を起動
2. 「辞書」タブから SKK 辞書ファイルを追加

**手動での設定**:
`~/.config/akaza/config.yml` の `skk_dicts` セクションに辞書パスを追加してください。

利用可能な SKK 辞書: https://skk-dev.github.io/dict/

## プロジェクト構成

このリポジトリは以下のクレートで構成されています：

| クレート | 説明 |
|----------|------|
| **ibus-akaza** | IBus エンジン本体 |
| **libakaza** | かな漢字変換エンジンのコアロジック |
| **akaza-conf** | GUI 設定ツール (GTK4) |
| **akaza-dict** | GUI 辞書編集ツール (GTK4) |
| **akaza-data** | 開発者向けツール（変換テスト、言語モデル生成、精度評価など） |
| **ibus-sys** | IBus の Rust バインディング |

### akaza-data の使い方

`akaza-data` は開発者向けのツールで、CLI からかな漢字変換のテストや言語モデルの生成ができます。

```bash
# かな漢字変換を実行
akaza-data check きょうはいいてんきですね
# => 今日/は/いい/天気/です/ね

# JSON 形式で複数候補を表示
akaza-data check --format json --candidates 3 きょうはいいてんきですね

# ユーザー学習データを使用
akaza-data check --user-data きょうはいいてんきですね

# 変換精度を評価
akaza-data evaluate --corpus corpus.txt --model-dir /path/to/model
```

詳細は `akaza-data --help` または [akaza-data/README.md](akaza-data/README.md) を参照してください。

## 開発

### 開発用ビルド・実行（推奨）

開発中は `dev-install` プロファイル（`opt-level=2`, `codegen-units=16`, `lto=false`）を使うことで、release ビルドより大幅に高速にビルドできます。

```bash
# 初回のみ: debug 用 xml をインストール（ibus が target/ のバイナリを直接起動する）
sudo make dev-setup

# 以降の開発サイクル: ビルド + ibus restart のみ（install 不要）
make dev-run
```

`dev-setup` を実行すると、IBus の component xml がビルドディレクトリのバイナリを指すようになるため、`sudo make install` でバイナリをコピーする必要がなくなります。

> **注意**: `sudo make install` を実行すると `target/` 内のファイルが root 所有になり、以降のビルドで権限エラーが発生します。ビルドとインストールは必ず分けて実行してください:
> ```bash
> make            # ユーザー権限でビルド
> sudo make install  # root 権限でインストール（ビルドは走らない）
> ```

### ビルド

```bash
# すべてのクレートをビルド
cargo build --workspace

# リリースビルド
cargo build --workspace --release

# 開発用ビルド（高速）
make dev

# テスト実行
cargo test --workspace
```

### コードフォーマット

```bash
cargo fmt --all
```

### Lint

```bash
cargo clippy -- -D warnings
```

## ライセンス

MIT License

## THANKS TO

* [ibus-uniemoji](https://github.com/salty-horse/ibus-uniemoji) を参考に初期の実装を行いました
* [日本語入力を支える技術](https://gihyo.jp/book/2012/978-4-7741-4993-6) を読み込んで実装しました。この本がなかったら実装しようと思わなかったと思います

### 言語リソース・ツール

デフォルト言語モデルの構築にあたり、以下の言語リソースを利用しています。各プロジェクトの開発者・提供者の皆様に感謝いたします。

* **[日本語 Wikipedia](https://ja.wikipedia.org/)** — コーパス統計データの主要な元データとして利用しています（CC BY-SA 4.0、Wikimedia Foundation および各執筆者）
* **[青空文庫](https://www.aozora.gr.jp/)** — コーパス統計データの元データとして利用しています（著作権消滅作品）
* **[CC-100 Japanese](https://data.statmt.org/cc-100/)** — コーパス統計データの補助データとして利用しています（Common Crawl 由来、Conneau et al., ACL 2020）
* **[Vibrato](https://github.com/daac-tools/vibrato)** — 形態素解析器として使用しています（Apache-2.0）
* **[IPADIC (MeCab)](https://taku910.github.io/mecab/)** — Vibrato の辞書としてモデルビルド時に使用しています（NAIST/ICOT BSD-style ライセンス、奈良先端科学技術大学院大学）
* **[UniDic CSJ](https://clrd.ninjal.ac.jp/unidic/)** — システム辞書の読み情報の補完に使用しています（BSD-3-Clause / GPL / LGPL、UniDic Consortium）
* **[SudachiDict](https://github.com/WorksApplications/SudachiDict)** — システム辞書の固有名詞データの補完に使用しています（Apache-2.0、WAQ LLC）
* **[SKK-JISYO.L](https://skk-dev.github.io/dict/)** — かな漢字変換の主辞書として利用しています。同梱はしておらず、ユーザーが別途インストールします（GPL-2.0）
* **[anthy-unicode](https://github.com/fujiwarat/anthy-unicode)** — 変換精度の評価コーパスとして利用しています（Public Domain）
