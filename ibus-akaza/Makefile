PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

PYTHON ?= /usr/bin/python3

all: akaza.xml ibus_akaza/config.py po/ibus-akaza.pot po/ja.mo

po/ibus-akaza.pot: ibus_akaza/ui.py ibus_akaza/input_mode.py
	xgettext -d ibus-akaza -o po/ibus-akaza.pot ibus_akaza/*.py

po/ja.mo: po/ja.po
	msgfmt -o po/ja.mo po/ja.po

install: ibus_akaza/config.py akaza.xml po/ja.mo
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza $(DESTDIR)$(SYSCONFDIR)/xdg/akaza $(DESTDIR)$(DATADIR)/ibus/component
	install -m 0755 -d $(DESTDIR)$(DATADIR)/locale

	install -m 0644 akaza.svg $(DESTDIR)$(DATADIR)/ibus-akaza
	install -m 0644 ibus.py $(DESTDIR)$(DATADIR)/ibus-akaza
	install -m 0644 ibus_akaza/ui.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 ibus_akaza/input_mode.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 ibus_akaza/config.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 ibus_akaza/keymap.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 ibus_akaza/config_loader.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 ibus_akaza/__init__.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 akaza.xml $(DESTDIR)$(DATADIR)/ibus/component

	install -m 0644 po/ja.mo $(DESTDIR)$(DATADIR)/locale/ja/LC_MESSAGES/ibus-akaza.mo

ibus_akaza/config.py: ibus_akaza/config.py.in
	sed -e "s:@SYSCONFDIR@:$(SYSCONFDIR):g" \
	    -e "s:@MODELDIR@:$(DESTDIR)/$(DATADIR)/akaza-data/:g" \
	    -e "s:@DICTIONARYDIR@:$(DESTDIR)/$(DATADIR)/ibus-akaza/dictionary:g" \
		$< > $@

akaza.xml: akaza.xml.in
	sed -e "s:@PYTHON@:$(PYTHON):g;" \
	    -e "s:@DATADIR@:$(DATADIR):g" $< > $@

test:
	python -m py_compile ibus.py
	python -m py_compile ibus_akaza/ui.py
	pytest

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza.svg
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/input_mode.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/ui.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/config.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/keymap.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/config_loader.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus.py
	rmdir $(DESTDIR)$(DATADIR)/ibus-akaza


clean:
	rm -f akaza.xml

.PHONY: all test install uninstall clean

