#!/bin/bash
# tagpr の command オプションから呼ばれるスクリプト。
# TAGPR_NEXT_VERSION 環境変数の値で全 Cargo.toml の version を更新する。
set -euo pipefail

VERSION="${TAGPR_NEXT_VERSION#v}"  # v プレフィックスを除去

if [ -z "$VERSION" ]; then
  echo "ERROR: TAGPR_NEXT_VERSION is not set" >&2
  exit 1
fi

echo "Updating Cargo.toml versions to ${VERSION}"

# workspace メンバーの Cargo.toml を更新
for toml in libakaza/Cargo.toml ibus-akaza/Cargo.toml ibus-sys/Cargo.toml akaza-data/Cargo.toml akaza-conf/Cargo.toml akaza-dict/Cargo.toml; do
  if [ -f "$toml" ]; then
    sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" "$toml"
    echo "  Updated $toml"
  fi
done
