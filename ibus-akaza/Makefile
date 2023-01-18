PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=
AKAZA_DATA_DIR ?= $(DATADIR)/akaza/

all: akaza.xml config.h

#   ibus_akaza/config.py: ibus_akaza/config.py.in
#   	sed -e "s:@SYSCONFDIR@:$(SYSCONFDIR):g" \
#   		-e "s:@MODELDIR@:$(DESTDIR)/$(DATADIR)/akaza-data/:g" \
#   		-e "s:@DICTIONARYDIR@:$(DESTDIR)/$(DATADIR)/ibus-akaza/dictionary:g" \
#   		$< > $@


config.h: config.h.in
	sed \
	    -e "s:@DATADIR@:$(DATADIR)/:g" $< > $@

akaza.xml: akaza.xml.in
	sed \
	    -e "s:@BINARY@:$(PREFIX)/bin/ibus-akaza:g" \
	    -e "s:@DATADIR@:$(DATADIR)/:g" $< > $@

akaza-debug.xml: akaza.xml.in
	sed \
	    -e "s:@BINARY@:${PWD}/ibus-akaza-debug.sh:g" \
	    -e "s:@DATADIR@:$(DATADIR)/:g" $< > $@

install: akaza.xml config.h
	cargo install --path=. --root=$(PREFIX)

	install -m 0644 akaza.svg $(DESTDIR)$(DATADIR)/ibus-akaza
	install -m 0644 akaza.xml $(DESTDIR)$(DATADIR)/ibus/component

install-debug: akaza-debug.xml
	install -m 0644 akaza-debug.xml $(DESTDIR)$(DATADIR)/ibus/component/akaza.xml

test:
	cargo test

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza.svg

clean:
	rm -f akaza.xml

.PHONY: all test install uninstall clean install-debug

