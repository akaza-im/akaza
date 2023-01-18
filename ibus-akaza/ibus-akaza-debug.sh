#!/bin/bash
set -ex

BASEDIR=$(dirname "$0")

umask 077

exec 1>> ~/.ibus-akaza.log
exec 2>&1

export AKAZA_DATA_DIR="$BASEDIR/../akaza-data/data/"
export AKAZA_ROMKAN_DIR="$BASEDIR/../romkan/"

export RUST_BACKTRACE=4

exec $BASEDIR/../target/release/ibus-akaza --ibus -vv
