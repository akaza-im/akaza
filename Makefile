PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share

install:
	install -m 0644 -v -D -t $(DATADIR)/akaza/romkan romkan/*
	install -m 0644 -v -D -t $(DATADIR)/akaza/keymap keymap/*
	$(MAKE) -C ibus-akaza install

