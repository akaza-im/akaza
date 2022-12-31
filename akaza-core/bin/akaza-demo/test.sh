#!/bin/sh
BASEDIR=$(dirname "$0")
DATADIR=$(readlink -f $BASEDIR/../../../akaza-data/data/)
RUST_BACKTRACE=1
set -ex
cargo run $DATADIR わたし
cargo run $DATADIR わたしのなまえはなかのです
