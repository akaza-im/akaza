# only really known to work on ubuntu, if you're using anything else, hopefully
# it should at least give you a clue how to install it by hand

PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

PYTHON ?= /usr/bin/python3

all: comb.xml config.py

check:
	python -m py_compile ibus.py
	python -m py_compile combromkan.py
	python -m py_compile comb.py
	python -m py_compile skkdict.py
	pytest

comb.xml: comb.xml.in
	sed -e "s:@PYTHON@:$(PYTHON):g;" \
	    -e "s:@DATADIR@:$(DATADIR):g" $< > $@

config.py: config.py.in
	sed -e "s:@SYSCONFDIR@:$(SYSCONFDIR):g" $< > $@

install: all check
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-comb $(DESTDIR)$(SYSCONFDIR)/xdg/comb $(DESTDIR)$(DATADIR)/ibus/component
	install -m 0644 comb.svg $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 ibus.py $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 combromkan.py $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 skkdict.py $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 comb.py $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 comb.xml $(DESTDIR)$(DATADIR)/ibus/component

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb.svg
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/config.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/ibus.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/skkdict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/combromkan.py
	rmdir $(DESTDIR)$(DATADIR)/ibus-comb
	rmdir $(DESTDIR)$(SYSCONFDIR)/xdg/comb
	rm -f $(DESTDIR)$(DATADIR)/ibus/component/comb.xml

clean:
	rm -f comb.xml
	rm -f config.py

.PHONY: all check install uninstall

