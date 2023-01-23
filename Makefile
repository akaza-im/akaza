PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share

install: install-resources
	$(MAKE) -C ibus-akaza install

install-resources:
	install -m 0644 -v -D -t $(DATADIR)/akaza/romkan romkan/*
	install -m 0644 -v -D -t $(DATADIR)/akaza/keymap keymap/*

.PHONY: install install-resources

