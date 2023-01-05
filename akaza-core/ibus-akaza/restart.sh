#!/bin/bash
set -x
cargo build
pkill -f target/debug/ibus-akaza
ibus engine akaza
