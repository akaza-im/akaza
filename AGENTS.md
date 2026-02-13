# Claude Code Development Guidelines

このファイルには、Claude Code を使用してこのプロジェクトを開発する際のガイドラインを記載します。

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
