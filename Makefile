PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share

all:
	$(MAKE) -C ibus-akaza all

install: install-resources
	$(MAKE) -C ibus-akaza install

install-resources:
	install -m 0644 -v -D -t $(DATADIR)/akaza/romkan romkan/*
	install -m 0644 -v -D -t $(DATADIR)/akaza/keymap keymap/*

clean:
	cargo clean
	$(MAKE) -C ibus-akaza clean

.PHONY: all install install-resources clean

