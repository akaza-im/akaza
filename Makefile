# only really known to work on ubuntu, if you're using anything else, hopefully
# it should at least give you a clue how to install it by hand

PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

PYTHON ?= /usr/bin/python3

all: comb.xml comb/config.py comb model/jawiki.1gram

check:
	python -m py_compile ibus.py
	python -m py_compile comb/combromkan.py
	python -m py_compile comb/engine.py
	python -m py_compile comb/skkdict.py
	pytest

comb.xml: comb.xml.in
	sed -e "s:@PYTHON@:$(PYTHON):g;" \
	    -e "s:@DATADIR@:$(DATADIR):g" $< > $@

comb/config.py: comb/config.py.in
	sed -e "s:@SYSCONFDIR@:$(SYSCONFDIR):g" \
	    -e "s:@MODELDIR@:$(DESTDIR)/$(DATADIR)/ibus-comb/model:g" \
	    -e "s:@DICTIONARYDIR@:$(DESTDIR)/$(DATADIR)/ibus-comb/dictionary:g" \
		$< > $@

model/system_language_model.trie: model/bin/create-system_language_model-from-json.py
	make -C model system_language_model.trie

model/system_dict.trie:
	make -C model system_dict.trie

install-dict: model/system_dict.trie
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-comb/dictionary
	install -p -m 0644 model/system_dict.trie $(DESTDIR)$(DATADIR)/ibus-comb/dictionary/

install: all comb/config.py model/jawiki.1gram install-dict
	install -m 0755 -d $(DESTDIR)$(DATADIR)/ibus-comb/comb $(DESTDIR)$(SYSCONFDIR)/xdg/comb $(DESTDIR)$(DATADIR)/ibus/component $(DESTDIR)$(DATADIR)/ibus-comb/model $(DESTDIR)$(DATADIR)/ibus-comb/dictionary
	install -m 0644 model/system_language_model.trie $(DESTDIR)$(DATADIR)/ibus-comb/model/

	install -m 0644 comb.svg $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 comb/__init__.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/graph.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/language_model.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/node.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/config.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/skkdict.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/combromkan.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 ibus.py $(DESTDIR)$(DATADIR)/ibus-comb
	install -m 0644 comb/engine.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/ui.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/user_language_model.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/system_language_model.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/system_dict.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb/user_dict.py $(DESTDIR)$(DATADIR)/ibus-comb/comb/
	install -m 0644 comb.xml $(DESTDIR)$(DATADIR)/ibus/component

uninstall:
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb.svg
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/config.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/engine.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/skkdict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/combromkan.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/graph.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/language_model.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/node.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/ui.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/user_language_model.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/system_language_model.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/user_dict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/comb/system_dict.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/ibus.py
	rm -f $(DESTDIR)$(DATADIR)/ibus-comb/model/system_language_model.trie
	rmdir $(DESTDIR)$(DATADIR)/ibus-comb
	rmdir $(DESTDIR)$(SYSCONFDIR)/xdg/comb
	rm -f $(DESTDIR)$(DATADIR)/ibus/component/comb.xml

clean:
	rm -f comb.xml
	rm -f comb/config.py

.PHONY: all check install uninstall

