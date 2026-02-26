#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODEL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
export PATH="$MODEL_DIR/../target/release:$PATH"

akaza-data tokenize-line \
    --system-dict "$MODEL_DIR/work/vibrato/ipadic-mecab-2_7_0/system.dic" \
    "$@"

