#!/bin/bash
set -ex

BASEDIR=$(dirname "$0")

exec 1>> ~/.ibus-akaza.log
exec 2>&1

export RUST_BACKTRACE=4
# export RUST_LOG=trace
export RUST_LOG=info

exec $BASEDIR/../target/debug/ibus-akaza --ibus
