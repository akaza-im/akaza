# Docker Testing Guide

このガイドでは、Dockerを使用してローカル環境でibus-akazaのテストを実行する方法を説明します。

## 前提条件

- Docker
- Docker Compose

## クイックスタート

### 1. Dockerイメージのビルド

```bash
make docker-test-build
```

### 2. テストの実行

#### すべてのテストを実行

```bash
make docker-test
```

#### Unit testsのみ実行

```bash
make docker-test-unit
```

#### Integration testsのみ実行

```bash
make docker-test-integration
```

#### E2E testsのみ実行

```bash
make docker-test-e2e
```

#### デバッグ用にシェルを起動

```bash
make docker-test-shell
```

シェル内では以下のコマンドが使用できます：

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration

# E2E tests (ignored testsを含む)
cargo test --test e2e -- --ignored --test-threads=1

# 特定のテスト
cargo test test_name
```

## Docker環境の詳細

### ベースイメージ

Ubuntu 24.04

### インストールされているツール

- **X11関連**: Xvfb, xdotool, dbus-x11, at-spi2-core
- **IBus**: ibus, libibus-1.0-dev
- **GTK4**: libgtk-4-dev, libgirepository1.0-dev
- **その他**: libmarisa-dev, clang, zstd
- **Rust**: 1.92.0 toolchain

### 環境変数

- `DISPLAY=:99`: 仮想ディスプレイ
- `DBUS_SESSION_BUS_ADDRESS=unix:path=/tmp/dbus-test-session`: D-Busセッションバス

## トラブルシューティング

### Dockerイメージが古い場合

イメージを再ビルドします：

```bash
docker-compose -f docker-compose.test.yml build --no-cache
```

### キャッシュをクリアしたい場合

Cargoのキャッシュとビルドキャッシュをクリア：

```bash
docker-compose -f docker-compose.test.yml down -v
```

### コンテナ内でデバッグしたい場合

シェルを起動してデバッグ：

```bash
make docker-test-shell

# コンテナ内で
cargo test --lib -- --nocapture
cargo test test_name -- --show-output
```

## CI/CDとの違い

GitHub Actions CIとDocker環境は同じ構成を使用しているため、ローカルでの動作がCIでも同じように動作することが期待できます。

主な違い:
- ローカル: Docker Composeでボリュームマウント
- CI: コードをチェックアウトして実行

## 参考

- GitHub Actions CI設定: `.github/workflows/ci-simple.yml`
- Dockerfile: `ibus-akaza/Dockerfile.test`
- Docker Compose設定: `docker-compose.test.yml`
