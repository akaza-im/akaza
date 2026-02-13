#!/bin/bash
set -ex

BASEDIR=$(dirname "$0")

umask 077

LOG_PATH="${HOME}/.cache/akaza/logs/ibus-akaza.log"
exec 1>> "$LOG_PATH"
exec 2>&1

export RUST_BACKTRACE=4

MEM_LIMIT_MB="${AKAZA_DEBUG_MEM_LIMIT_MB:-2048}"
if ! ulimit -v "$((MEM_LIMIT_MB * 1024))"; then
  echo "warning: failed to set ulimit -v (MEM_LIMIT_MB=${MEM_LIMIT_MB})" >&2
fi

# dev-install プロファイルのバイナリがあればそちらを優先、なければ release を使う
if [ -x "$BASEDIR/../target/dev-install/ibus-akaza" ] && \
   [ "$BASEDIR/../target/dev-install/ibus-akaza" -nt "$BASEDIR/../target/release/ibus-akaza" ] 2>/dev/null; then
  exec "$BASEDIR/../target/dev-install/ibus-akaza" --ibus -vv
else
  exec "$BASEDIR/../target/release/ibus-akaza" --ibus -vv
fi
