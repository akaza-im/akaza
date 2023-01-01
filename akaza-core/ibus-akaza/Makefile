PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

all: akaza.xml

#   ibus_akaza/config.py: ibus_akaza/config.py.in
#   	sed -e "s:@SYSCONFDIR@:$(SYSCONFDIR):g" \
#   		-e "s:@MODELDIR@:$(DESTDIR)/$(DATADIR)/akaza-data/:g" \
#   		-e "s:@DICTIONARYDIR@:$(DESTDIR)/$(DATADIR)/ibus-akaza/dictionary:g" \
#   		$< > $@

akaza.xml: akaza.xml.in
	sed \
	    -e "s:@DATADIR@:$(DATADIR):g" $< > $@


install: ibus_akaza/config.py akaza.xml po/ja.mo
	cargo install
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza $(DESTDIR)$(SYSCONFDIR)/xdg/akaza $(DESTDIR)$(DATADIR)/ibus/component
	install -m 0755 -d $(DESTDIR)$(DATADIR)/locale

	install -m 0644 akaza.svg $(DESTDIR)$(DATADIR)/ibus-akaza
	install -m 0644 akaza.xml $(DESTDIR)$(DATADIR)/ibus/component

test:
	cargo test

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza.svg

clean:
	rm -f akaza.xml

.PHONY: all test install uninstall clean

