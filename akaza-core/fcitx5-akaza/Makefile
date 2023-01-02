PREFIX?=/usr/
FCITX_DIR?=$(PREFIX)/share/fcitx5
ADDON_DIR=$(FCITX_DIR)/addon
IM_DIR=$(FCITX_DIR)/inputmethod
FCITX_LIB_DIR?=$(PREFIX)/lib/fcitx5/

all: build

build: ../target/release/libfcitx5_akaza.so

../target/release/libfcitx5_akaza.so: build.rs src/fcitx5-akaza.cc src/fcitx5-akaza.h
	cargo build --release

install: build
	mkdir -p "$(ADDON_DIR)"
	mkdir -p "$(IM_DIR)"
	mkdir -p "$(FCITX_LIB_DIR)"
	cp files/akaza-addon.conf $(ADDON_DIR)
	cp files/akaza.conf $(IM_DIR)
	# strip ../target/release/libfcitx5_akaza.so
	cp ../target/release/libfcitx5_akaza.so "$(FCITX_LIB_DIR)"/libfcitx5-akaza.so

test:
	cargo test

clean:
	cargo clean

.PHONY: all test clean install
