#!/bin/bash
set -x
LOG_PATH="${HOME}/.cache/akaza/logs/ibus-akaza.log"

cargo build --release || { echo 'cannot build.' ; exit 1; }
ibus restart
ibus engine akaza
tail -F "$LOG_PATH"
