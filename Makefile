# only really known to work on ubuntu, if you're using anything else, hopefully
# it should at least give you a clue how to install it by hand

PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

PYTHON ?= /usr/bin/python3

all: comb.xml comb

check:
	python -m py_compile ibus.py
	python -m py_compile comb/combromkan.py
	python -m py_compile comb/comb.py
	python -m py_compile comb/skkdict.py
	pytest

comb.xml: comb.xml.in
	sed -e "s:@PYTHON@:$(PYTHON):g;" \
	    -e "s:@DATADIR@:$(DATADIR):g" $< > $@

comb/config.py: comb/config.py.in
	sed -e "s:@SYSCONFDIR@:$(SYSCONFDIR):g" $< > $@

install: all check comb/config.py
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-comb/comb $(DESTDIR)$(SYSCONFDIR)/xdg/comb $(DESTDIR)$(DATADIR)/ibus/component
	install -m 0644 comb.svg $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 comb/__init__.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/graph.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/skkdict.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/combromkan.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 ibus.py $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 comb/comb.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb.xml $(DESTDIR)$(DATADIR)/ibus/component

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb.svg
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/config.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/comb.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/skkdict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/combromkan.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/graph.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/ibus.py
	rmdir $(DESTDIR)$(DATADIR)/ibus-comb
	rmdir $(DESTDIR)$(SYSCONFDIR)/xdg/comb
	rm -f $(DESTDIR)$(DATADIR)/ibus/component/comb.xml

clean:
	rm -f comb.xml
	rm -f comb/config.py

.PHONY: all check install uninstall

