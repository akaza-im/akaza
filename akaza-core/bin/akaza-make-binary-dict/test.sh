#!/bin/bash
BASEDIR=$(dirname "$0")
DATADIR=$BASEDIR/../../../akaza-data/
cargo run $DATADIR/work/jawiki.system_dict.txt $DATADIR/data/system_dict.cdb
akaza-make-binary-dict $DATADIR/work/jawiki.single_term.txt $DATADIR/data/single_term.cdb

