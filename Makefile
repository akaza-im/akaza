PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share

build:
	cargo build --release -p ibus-akaza -p akaza-conf -p akaza-dict -p akaza-data

# 開発用ビルド: release より高速（opt-level=2, codegen-units=16, lto=false）
dev:
	cargo build --profile dev-install -p ibus-akaza -p akaza-conf -p akaza-dict -p akaza-data

all: build
	$(MAKE) -C ibus-akaza all

# install はビルド済みバイナリのコピーのみ行う。
# ビルドは事前に `make` で実行しておくこと。
#   make && sudo make install
# sudo で cargo build が走って target/ が root 所有になるのを防ぐ。
install: install-resources install-model
	install -m 0755 target/release/ibus-akaza $(PREFIX)/bin/
	install -m 0755 target/release/akaza-conf $(PREFIX)/bin/
	install -m 0755 target/release/akaza-dict $(PREFIX)/bin/
	install -m 0755 target/release/akaza-data $(PREFIX)/bin/
	$(MAKE) -C ibus-akaza install

# 開発用: ビルド + ibus restart のみ（install 不要）
# 初回は make dev-setup で debug 用 xml をインストールしておくこと
dev-run: dev
	ibus restart

# 開発環境の初期セットアップ: debug 用 xml をインストール
# ibus が target/ のバイナリを直接起動するようになる
dev-setup: install-resources
	$(MAKE) -C ibus-akaza akaza-debug.xml
	$(MAKE) -C ibus-akaza install-debug

install-model:
	mkdir -p $(DATADIR)/akaza/model/default/
	curl -L https://github.com/akaza-im/akaza-default-model/releases/latest/download/akaza-default-model.tar.gz | \
		tar xzv --strip-components=1 -C $(DATADIR)/akaza/model/default/

install-resources:
	install -m 0644 -v -D -t $(DATADIR)/akaza/romkan romkan/*
	install -m 0644 -v -D -t $(DATADIR)/akaza/keymap keymap/*

clean:
	cargo clean
	$(MAKE) -C ibus-akaza clean

# Docker test targets
docker-test-build:
	docker compose -f docker-compose.test.yml build

docker-test:
	docker compose -f docker-compose.test.yml run --rm test test

docker-test-unit:
	docker compose -f docker-compose.test.yml run --rm test test-unit

docker-test-integration:
	docker compose -f docker-compose.test.yml run --rm test test-integration

docker-test-e2e:
	docker compose -f docker-compose.test.yml run --rm test test-e2e

docker-test-shell:
	docker compose -f docker-compose.test.yml run --rm test bash

docs-build:
	cd docs && mdbook build

docs-serve:
	cd docs && mdbook serve --open

.PHONY: all build dev dev-run dev-setup install install-model install-resources clean \
	docker-test-build docker-test docker-test-unit docker-test-integration docker-test-e2e docker-test-shell \
	docs-build docs-serve
