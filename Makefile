# only really known to work on ubuntu, if you're using anything else, hopefully
# it should at least give you a clue how to install it by hand

PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

PYTHON ?= /usr/bin/python3

test:
	cd akaza-data && pytest
	cd akaza-core && pytest
	cd ibus-akaza && pytest

install:
	cd akaza-data && $(PYTHON) setup.py install
	cd akaza-core && $(PYTHON) setup.py install

#install: all akaza/config.py model/system_dict.trie install-data
#	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-akaza/akaza $(DESTDIR)$(SYSCONFDIR)/xdg/akaza $(DESTDIR)$(DATADIR)/ibus/component $(DESTDIR)$(DATADIR)/ibus-akaza/model $(DESTDIR)$(DATADIR)/ibus-akaza/dictionary
#
#	install -m 0644 akaza.svg $(DESTDIR)$(DATADIR)/ibus-akaza
#	install -m 0644 ibus.py $(DESTDIR)$(DATADIR)/ibus-akaza
#	install -m 0644 akaza.xml $(DESTDIR)$(DATADIR)/ibus/component
#
#	install -m 0644 akaza/__init__.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/graph.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/language_model.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/node.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/config.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/skkdict.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/romkan.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/engine.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/ui.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/user_language_model.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/system_language_model.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/system_dict.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/
#	install -m 0644 akaza/user_dict.py $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza
	rm -f $(DESTDIR)$(DATADIR)/ibus/component/akaza.xml

	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/engine.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/skkdict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/akazaromkan.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/graph.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/language_model.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/node.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/ui.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/user_language_model.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/system_language_model.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/user_dict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/akaza/system_dict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-akaza/model/system_language_model.trie
	rmdir $(DESTDIR)$(SYSCONFDIR)/xdg/akaza

clean:
	rm -f akaza.xml
	rm -f akaza/config.py

.PHONY: all install uninstall test
