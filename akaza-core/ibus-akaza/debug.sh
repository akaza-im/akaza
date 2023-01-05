#!/bin/bash
set -x
cargo build
ibus restart
ibus engine akaza
tail -F ~/.ibus-akaza.log
