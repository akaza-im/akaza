#!/bin/bash
set -ex

BASEDIR=$(dirname "$0")

umask 077

exec 1>> ~/.ibus-akaza.log
exec 2>&1

export RUST_BACKTRACE=4

exec $BASEDIR/../target/debug/ibus-akaza --ibus -vvvvv
