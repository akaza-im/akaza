#!/bin/bash
set -x
cargo build --release || { echo 'cannot build.' ; exit 1; }
ibus restart
ibus engine akaza
tail -F ~/.ibus-akaza.log
