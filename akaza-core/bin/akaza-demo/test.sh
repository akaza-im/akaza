#!/bin/sh
BASEDIR=$(dirname "$0")
DATADIR=$(readlink -f $BASEDIR/../../../akaza-data/data/)
set -ex
cargo run $DATADIR わたし
cargo run $DATADIR わたしのなまえはなかのです
