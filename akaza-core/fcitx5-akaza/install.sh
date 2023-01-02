#!/bin/bash
set -ex

if [ -z "$var" ]
then
    FCITX5_BASE=/usr/share/fcitx5
fi

VERSION=$(perl -ne 'print $1 if /version = "(.*)"/' Cargo.toml)
echo VERSION=$VERSION

cargo build

cat akaza.conf.in > akaza.conf
VERSION=$VERSION perl -pe 's/<<<PROJECT_VERSION>>>/$ENV{VERSION}/g' akaza-addon.conf.in > akaza-addon.conf

sudo install -m 0755 akaza-addon.conf "$FCITX5_BASE/addon/akaza-addon.conf"
sudo install -m 0755 akaza.conf "$FCITX5_BASE/inputmethod/akaza.conf"
sudo install -m 0755 ../target/debug/libfcitx5_akaza.so /usr/lib/fcitx5/libakaza.so


