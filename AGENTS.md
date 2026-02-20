# Claude Code Development Guidelines

このファイルには、Claude Code を使用してこのプロジェクトを開発する際のガイドラインを記載します。

**注意**: `CLAUDE.md` は `AGENTS.md` への symlink です。実体は `AGENTS.md` にあるので、編集時は `AGENTS.md` を変更してください。

## コミット前のチェックリスト

### 必須: コードフォーマット

**コミット前に必ず `cargo fmt` を実行してください。**

```bash
cargo fmt
```

これにより、Rust コードが統一されたスタイルでフォーマットされます。
PR 作成前にも必ず実行してください。

### 推奨: テスト実行

変更後にテストを実行して、既存機能が壊れていないことを確認してください。

#### Docker環境でのテスト（推奨）

ibus-akazaのテストはX11とIBusが必要なため、Docker環境での実行を推奨します：

```bash
# Dockerイメージのビルド（初回のみ）
make docker-test-build

# すべてのテストを実行
make docker-test

# Unit testsのみ
make docker-test-unit

# Integration testsのみ
make docker-test-integration

# E2E testsのみ（将来拡張予定）
make docker-test-e2e

# デバッグ用シェル
make docker-test-shell
```

Docker環境はGitHub Actions CIと同じ構成を使用しているため、ローカルでの動作がCIでも同じように動作します。

詳細は `ibus-akaza/DOCKER_TESTING.md` を参照してください。

#### 直接実行（libakaza等、X11不要なパッケージ）

```bash
# 全体のテスト
cargo test

# 特定のパッケージのみ
cargo test --package libakaza
cargo test --package akaza-data

# ibus-akazaのunit testsのみ（X11不要）
cargo test --lib --package ibus-akaza
```

### 推奨: Clippy チェック

コードの品質を確認するため、clippy を実行してください。

```bash
cargo clippy --all-targets --all-features
```

## コミットメッセージ

- 日本語または英語で記述
- 変更内容を簡潔に説明
- 複数の変更がある場合は箇条書きで記載
- Co-Authored-By を含める（自動化されている場合）

## ブランチ戦略

- `main`: 安定版ブランチ
- 機能追加やバグ修正は feature ブランチから PR を作成
- ブランチ名は内容が分かるように命名（例: `add-core-tests`, `fix-rsmarisa-migration`）

## PR 作成時

- **タイトルと本文は日本語で記述する**
- タイトルは変更内容を明確に
- **本文は日本語で記述すること** （このプロジェクトの方針）
- 本文には以下を含める：
  - Summary: 変更の概要
  - 変更内容の詳細
  - テスト結果（Docker環境での実行結果を含む）
  - 関連する Issue があれば記載（例: `Related: #354`）
- コメントも日本語で記述する

## 開発フロー

1. main ブランチから最新版を取得: `git checkout main && git pull`
2. 新しいブランチを作成: `git checkout -b feature-name`
3. 変更を実装
4. **`cargo fmt` を実行** ← 重要！
5. テストを実行: `cargo test`
6. 変更をコミット: `git add -A && git commit -m "..."`
7. プッシュ: `git push -u origin feature-name`
8. PR を作成: `gh pr create ...`

## プロジェクト固有の注意事項

### テストについて

- libakaza の変更時は必ずテストを追加または更新
- 新機能には対応するテストを追加
- エッジケースのテストも考慮
- **テストが書きにくい箇所を見つけた場合**：
  - リファクタリングの機会として Issue に登録
  - テスタビリティ向上のための設計改善を検討
  - 依存関係が多すぎる、モックが困難などの問題を記録

#### テストのレイヤー

1. **Unit Tests**: 個別の関数やモジュールのテスト
   - FFIをモック化
   - 高速（<1秒）
   - `cargo test --lib` で実行

2. **Integration Tests**: モジュール間の連携テスト
   - IBusなしで状態遷移を検証
   - 中速（~10秒）
   - `cargo test --test integration` で実行

3. **E2E Tests**: 実際のIBusを使用したテスト
   - 実際のXvfb + IBusが必要
   - 低速（~60秒）
   - `cargo test --test e2e -- --ignored --test-threads=1` で実行
   - Docker環境推奨: `make docker-test-e2e`

#### テストが書きにくいコードについて

テストが書きにくいコードを発見した場合:

1. Issue を作成して記録
2. 以下の情報を含める:
   - なぜテストが書きにくいか
   - どのような依存関係があるか
   - リファクタリングの提案（あれば）
3. ラベル `testability` を付ける

これにより、将来的なリファクタリングの参考になります。

例: Issue #353 - CurrentState のテスト容易性改善

### 依存関係の更新

- renovate が自動的に依存関係を更新
- 重要な更新は手動で確認

### ドキュメント

- README.md は最新の状態に保つ
- コード内のコメントは日本語で記述可能
- 複雑なロジックには説明コメントを追加

---

## default-model/ — デフォルト言語モデル生成

`default-model/` ディレクトリは、Akaza のデフォルト言語モデルとシステム辞書を生成する。corpus-stats の統計データをダウンロードし、コーパス補正を適用してモデルを訓練し、marisa-trie 形式のモデルを生成する。

### ビルド

```bash
# akaza-data をローカルビルド (default-model/Makefile が自動的に ../target/release を PATH に追加)
cargo build --release -p akaza-data

# モデルビルド (初回は corpus-stats tarball をダウンロード)
make -C default-model

# 評価
make -C default-model evaluate

# ルートからの convenience target
make model
make evaluate
```

### パイプライン

1. **corpus-stats ダウンロード** — akaza-corpus-stats GitHub Release → `default-model/work/`
2. **モデル訓練** — `learn-corpus` でコーパス補正適用 → `default-model/data/unigram.model`, `default-model/data/bigram.model`
3. **システム辞書構築** — vocab + corpus + UniDic → `default-model/data/SKK-JISYO.akaza`
4. **評価** — anthy-corpus テストセットで評価 (corpus.4.txt は除外)

### 評価スコア履歴

`default-model/README.md` の「評価スコア履歴」セクションに、`make evaluate` による再現率の推移が記録されている。モデル変更時はここを参照して退行がないか確認すること。

### 変換テスト（akaza-data check）

`akaza-data check` で CLI からかな漢字変換の動作を確認できる。

```bash
# 基本的な変換テスト
cargo run --release -p akaza-data -- check "ごじょうほう"
# => ご/情報

# 各文節の候補を表示（-n で候補数指定）
cargo run --release -p akaza-data -- check -n 3 "ごじょうほう"

# JSON 形式で候補とコストを表示（分節・候補の詳細確認に便利）
cargo run --release -p akaza-data -- check -n 3 -f json "ごじょうほう"

# k-best: 上位 k 個の分節パターンをスコア付きで表示
cargo run --release -p akaza-data -- check -k 3 "ごじょうほう"
```

### データフォーマット

#### default-model/training-corpus/*.txt (学習コーパス)

`漢字/よみ` のスペース区切り。`;; ` で始まる行はコメント。

```
僕/ぼく の/の 主観/しゅかん では/では そう/そう です/です
```

Three tiers with different training epoch counts:
- **must.txt** — Must convert correctly (10,000 epochs). Shipping quality gate.
- **should.txt** — Should convert correctly (100 epochs). Send PRs here.
- **may.txt** — Nice to have (10 epochs).

Words in corpus files are automatically registered in the system dictionary. Delta parameter (2000) controls corpus influence strength.

**重要**: コーパスの単語境界は vibrato (ipadic) のトークナイズ結果に合わせること。ただし読みは vibrato の出力を鵜呑みにせず、文脈に合った正しい読みを書くこと。vibrato は「行って」を「おこなって」、「日本」を「にっぽん」と読むなど、文脈を無視した読みを返すことがある。`default-model/scripts/tokenize-line.sh` で単語境界を確認し、読みは自分で正しく付ける。vibrato の辞書は `default-model/work/vibrato/ipadic-mecab-2_7_0/system.dic` にある（`make -C default-model` 実行後に生成される）。

**重要**: bigram モデルは BOS（文頭）・EOS（文末）をトークンとして使用するため、コーパスには原則として完全な文を追加すること。

```bash
# 単一文の確認
./default-model/scripts/tokenize-line.sh "買い物に行ってくる"
# => 買い物/かいもの に/に 行って/おこなって くる/くる
# ※ 単語境界(4トークン)は正しいが、読み「おこなって」は誤り
# ※ コーパスでは: 買い物/かいもの に/に 行って/いって くる/くる
```

#### default-model/dict/SKK-JISYO.akaza

SKK dictionary format. For vocabulary not in SKK-JISYO.L.

```
きめつのやいば /鬼滅の刃/
かなにゅうりょく /かな入力/仮名入力/
```

Format: `よみ /候補1/候補2/.../`

#### default-model/anthy-corpus/*.txt (評価用コーパス)

Evaluation data from anthy-unicode. Each line is pipe-delimited reading + expected kanji pair. corpus.4.txt is excluded from evaluation (contains known error cases).

**注意**: anthy コーパスの表記基準に合わせる必要はない。表記スタイルの違い（ください/下さい、ない/無い 等）は誤変換ではない。

### Evaluate フィルタリング

`default-model/evaluate-filter/` に評価結果のフィルタリング定義がある:

- **`accept.tsv`** — `入力読み\t許容するakaza出力\tcorpus期待値\t理由` 形式
- **`ignore.txt`** — 評価から除外する入力
- **`scripts/filter-evaluate.py`** — bad.txt をフィルタして Real BAD を算出

accept.tsv に入れてよいもの:
- 漢字↔ひらがな のスタイル差（綺麗/きれい、沢山/たくさん、出来る/できる 等）
- カタカナ↔ひらがな のスタイル差（ダメ/だめ、アホ/あほ 等）
- corpus 側が怪しいケース（再製紙→再生紙 等）

accept.tsv に入れてはいけないもの:
- Wikipedia 由来の珍語が勝っているケース → 修正すべき本当のバグ
- 明らかに日本語として不自然な出力
- **補助動詞のひらがな→漢字変換**: 「やってみる→やって見る」は誤り
- **送り仮名の省略が不自然なケース**

### BAD エントリの分類方法

1. **表記揺れ**（→ accept.tsv）: どちらも日本語として正しいケース
2. **口語分節崩壊**（→ should.txt）: 口語表現が別の漢字列に化ける
3. **bigram 不足**（→ should.txt）: 隣接単語ペアの共起スコア不足
4. **Wikipedia 偏り**（→ should.txt）: 専門用語が日常語のスコアを上回る
5. **熟語不足**（→ should.txt or dict/SKK-JISYO.akaza）
6. **wordplay・意味不明**（→ ignore.txt）
7. **corpus 側の問題**（→ accept.tsv with corpus_wrong）

### コーパス育成の方針

#### should.txt に追加すべきパターン
- **一方向の同音異義語**: 逆方向の誤変換が少ないもの
- **Wikipedia 偏りの矯正**: 日常語が Wikipedia 用語に負けるケース
- **分節崩壊の修正**: 口語表現の正しい分節
- **珍妙変換**: 一般表現が古典漢字に化けるケース

#### may.txt 追加時の危険パターン
- **助詞と同じ読みを持つ漢字**: 煮/に、荷/に 等。大量退行を引き起こす
- **高頻度基本語と同じ読み**: 気/木、見/診 等。数十件単位の退行が発生する
- **退行チェック**: 追加後は必ず evaluate を実行すること

### チューニング知見

- **双方向同音異義語の罠**: `各/書く/核` 等は should.txt での調整が困難。辞書の複合語エントリで対処
- **退行チェックの必須化**: evaluate 後 sort して diff すること
- **珍妙パターン**: Wikipedia 由来の歴史人物名や古典漢字が高スコアになる場合、辞書登録+コーパスで対処
- **vocab 由来の熟字訓・難読語**: corpus-stats の vocab 生成（threshold=16）では、読みの文字数と表層形の文字数の乖離をチェックしていない。そのため Vibrato が出力する熟字訓・難読語がそのまま辞書に入る。以下は読みよりも漢字の方が文字数が多いエントリの例:
  - 2.0x: `ご → 豆汁/豆油`
  - 1.5x: `ぐみ → 胡頽子`、`けら → 啄木鳥`、`しめ → 七五三`、`びん → 保栄茂`、`ふし → 五倍子`、`ぶな → 山毛欅`、`もず → 百舌鳥`
  - 1.06〜1.25x: 中黒（・）を含む企業名・新聞名（ル・マン後、アイ・オー・データ機器 等）

### デフォルトモデルの Release

アプリの `v*` タグ push 時に GitHub Actions がモデルビルド+評価を実行し、`akaza-default-model.tar.gz` を同じ GitHub Release に添付する。

---

## corpus-stats/ — コーパス統計データ生成

`corpus-stats/` ディレクトリは、Akaza 用の n-gram 統計データを生成するパイプライン。日本語 Wikipedia (CirrusSearch ダンプ)、青空文庫、CC-100 Japanese をトーカナイズし、unigram/bigram の wordcnt trie と語彙ファイルを生成する。

### ビルド

```bash
# git submodule の初期化 (青空文庫テキスト)
git submodule update --init corpus-stats/aozorabunko_text

# akaza-data をローカルビルド
cargo build --release -p akaza-data

# ビルド (jawiki + 青空文庫のみ)
make -C corpus-stats

# CC-100 込みビルド
make -C corpus-stats all-full

# 配布用成果物の生成
make -C corpus-stats dist        # dist/ (jawiki + 青空文庫)
make -C corpus-stats dist-full   # dist-full/ (jawiki + 青空文庫 + CC-100)

# ルートからの convenience target
make corpus-stats
```

### Makefile 変数

- `CIRRUS_DATE`: Wikipedia ダンプの日付 (デフォルト: `20251229`)
- `CC100_LIMIT`: CC-100 の処理文書数上限 (デフォルト: `5000000`、`0` で無制限)
- `TOKENIZER_OPTS`: `akaza-data tokenize` への追加オプション

### パイプライン

```
Wikipedia CirrusSearch (.json.gz)
    → extract-cirrus.py → corpus-stats/work/jawiki/extracted/
    → akaza-data tokenize → corpus-stats/work/jawiki/vibrato-ipadic/

青空文庫 (corpus-stats/aozorabunko_text submodule)
    → akaza-data tokenize → corpus-stats/work/aozora_bunko/vibrato-ipadic/

CC-100 Japanese (ja.txt.xz)
    → extract-cc100.py → corpus-stats/work/cc100/extracted/
    → akaza-data tokenize → corpus-stats/work/cc100/vibrato-ipadic/

統計生成:
    jawiki + aozora       → wfreq → vocab / unigram.trie / bigram.trie → corpus-stats/dist/
    jawiki + aozora + cc100 → *-full → corpus-stats/dist-full/
```

### Key Files

- `corpus-stats/Makefile` — ビルドパイプライン全体の定義
- `corpus-stats/scripts/extract-cirrus.py` — CirrusSearch NDJSON → `<doc>` 形式変換
- `corpus-stats/scripts/extract-cc100.py` — CC-100 → `<doc>` 形式変換 (品質フィルタ付き)
- `corpus-stats/mecab-user-dict.csv` — Vibrato ユーザー辞書
- `corpus-stats/NOTICE` — 生成データのライセンス情報

### CC-100 フィルタ

`corpus-stats/scripts/extract-cc100.py` は文書単位で以下のフィルタを適用:
1. 最小文書長 (200 文字未満を除外)
2. ひらがな比率 (10% 未満を除外)
3. 行の繰り返し (30% 以上重複行で除外)
4. 制御文字・私用領域の除去

### corpus-stats の Release

CalVer (`YYYY.MMDD.PATCH`)。ローカルビルド + `gh` CLI で `make -C corpus-stats release` を実行。
