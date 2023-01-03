#!/bin/bash
set -ex

BASEDIR=$(dirname "$0")

exec 1>> ~/.ibus-akaza.log
exec 2>&1

export RUST_BACKTRACE=1
export RUST_LOG=trace

exec $BASEDIR/../target/debug/ibus-akaza --ibus
