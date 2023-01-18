PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share

install:
	install -m 0644 -v -D -t $(DATADIR)/akaza/romkan romkan/*
	$(MAKE) -C ibus-akaza install

