PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

PYTHON ?= /usr/bin/python3

all: akaza.xml ibus_akaza/config.py

install: ibus_akaza/config.py akaza.xml
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza $(DESTDIR)$(SYSCONFDIR)/xdg/akaza $(DESTDIR)$(DATADIR)/ibus/component

	install -m 0644 akaza.svg $(DESTDIR)$(DATADIR)/ibus-akaza
	install -m 0644 ibus.py $(DESTDIR)$(DATADIR)/ibus-akaza
	install -m 0644 ibus_akaza/ui.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 ibus_akaza/config.py $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/
	install -m 0644 akaza.xml $(DESTDIR)$(DATADIR)/ibus/component

ibus_akaza/config.py: ibus_akaza/config.py.in
	sed -e "s:@SYSCONFDIR@:$(SYSCONFDIR):g" \
	    -e "s:@MODELDIR@:$(DESTDIR)/$(DATADIR)/ibus-akaza/model:g" \
	    -e "s:@DICTIONARYDIR@:$(DESTDIR)/$(DATADIR)/ibus-akaza/dictionary:g" \
		$< > $@

akaza.xml: akaza.xml.in
	sed -e "s:@PYTHON@:$(PYTHON):g;" \
	    -e "s:@DATADIR@:$(DATADIR):g" $< > $@

check:
	python -m py_compile ibus.py
	python -m py_compile ibus_akaza/ui.py

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza.svg
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/ui.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus_akaza/config.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/ibus.py
	rmdir $(DESTDIR)$(DATADIR)/ibus-akaza


clean:
	rm -f akaza.xml

.PHONY: all check install uninstall clean

