PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=
AKAZA_DATA_DIR ?= $(DATADIR)/akaza/

MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
MKFILE_DIR := $(dir $(MKFILE_PATH))

all: akaza.xml config.h ../target/release/ibus-akaza

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
	    -e "s:@BINARY@:$(MKFILE_DIR)/ibus-akaza-debug-engine.sh:g" \
	    -e "s:@DATADIR@:$(DATADIR)/:g" $< > $@

# install はファイルコピーのみ。ビルドは事前に済ませておくこと。
install: akaza.xml config.h
	install -m 0755 ../target/release/ibus-akaza $(PREFIX)/bin/

	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-akaza
	install -m 0644 akaza.svg $(DESTDIR)$(DATADIR)/ibus-akaza/
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus/component
	install -m 0644 akaza.xml $(DESTDIR)$(DATADIR)/ibus/component/

install-debug: akaza-debug.xml
	install -m 0644 akaza-debug.xml $(DESTDIR)$(DATADIR)/ibus/component/akaza.xml

test:
	cargo test

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza.svg

clean:
	rm -f akaza.xml config.h

.PHONY: all test install uninstall clean install-debug
