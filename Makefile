# only really known to work on ubuntu, if you're using anything else, hopefully
# it should at least give you a clue how to install it by hand

PREFIX ?= /usr
SYSCONFDIR ?= /etc
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

PYTHON ?= /usr/bin/python3

test:
	cd pyakaza && pytest tests
	cd ibus-akaza && pytest

install:
	cd libakaza && cmake . && make && make install
	cd akaza-data && make install
	cd pyakaza && make clean && $(PYTHON) setup.py install
	cd ibus-akaza && make install

.PHONY: all install uninstall test
