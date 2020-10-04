PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share
DESTDIR ?=

CXX=clang++
CXXOPTS=-Wall -g -std=c++20
# Note: `shell` is a gnu make extension.
EXT_SUFFIX:=$(shell python3-config --extension-suffix)

all: akaza_data/data/system_dict.trie akaza_data/data/lm_v2_2gram.trie akaza_data/data/lm_v2_2gram.trie binary akaza_data/data/system_dict.trie
binary: akaza_data/systemlm_loader$(EXT_SUFFIX)

work/jawiki-latest-pages-articles.xml.bz2:
	mkdir -p work/
	wget --no-verbose --no-clobber -O work/jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

work/jawiki-latest-pages-articles.xml: work/jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep work/jawiki-latest-pages-articles.xml.bz2

work/extracted/BE/wiki_13: work/jawiki-latest-pages-articles.xml
	python -m wikiextractor.WikiExtractor --quiet --processes 8 --out work/extracted/ work/jawiki-latest-pages-articles.xml

work/annotated/BE/wiki_13: work/extracted/BE/wiki_13 bin/010_wiki2annotated.py
	python bin/010_wiki2annotated.py

work/text/BE/wiki_13: work/annotated/BE/wiki_13 bin/020_annotated2text.py bin/akaza_data_utils/merge_terms.py
	python bin/020_annotated2text.py

work/jawiki.wfreq: work/text/BE/wiki_13 bin/030_text2wfreq.py
	python bin/030_text2wfreq.py

# 118,334=entry count of libkkc-data.
work/jawiki.vocab: work/jawiki.wfreq bin/040_wfreq2vocab.py
	python bin/040_wfreq2vocab.py

work/jawiki.1gram.json: work/jawiki.vocab  bin/050_dumpngram.py
	python bin/050_dumpngram.py work/jawiki.vocab

work/jawiki.merged-2gram.txt: work/jawiki.1gram.json work/jawiki.2gram.json work/jawiki.vocab bin/060_create-system_language_model.py
	python bin/060_create-system_language_model.py

work/jawiki.single_dict.txt: work/jawiki.vocab jawiki-kana-kanji-dict/SKK-JISYO.jawiki
	python bin/070_make-system-dict.py
	python bin/080_system_dump.py

bin/090_effective_lm: bin/090_effective_lm.cc
	$(CXX) $(CXXOPTS) -l marisa bin/090_effective_lm.cc -o bin/090_effective_lm

bin/100_effective_dict: bin/100_effective_dict.cc
	$(CXX) $(CXXOPTS) -l marisa bin/100_effective_dict.cc -o bin/100_effective_dict

akaza_data/data/lm_v2_2gram.trie: bin/090_effective_lm work/jawiki.merged-2gram.txt
	./bin/090_effective_lm

akaza_data/data/system_dict.trie: bin/100_effective_dict work/jawiki.single_dict.txt
	./bin/100_effective_dict

# -------------------------------------------------------------------------

akaza_data/systemlm_loader$(EXT_SUFFIX): akaza_data/systemlm_loader.cc src/system_lm.h src/binary_dict.h src/nanoutf8.h
	$(CXX) -O3 -Wall -shared -lmarisa -std=c++11 -fPIC `python3 -m pybind11 --includes` akaza_data/systemlm_loader.cc \
		-o akaza_data/systemlm_loader$(EXT_SUFFIX)

# -------------------------------------------------------------------------

t/00_systemlm: t/00_systemlm.cc
	$(CXX) -std=c++17 -l marisa -o t/00_systemlm t/00_systemlm.cc

t/01_binary_dict: t/01_binary_dict.cc src/binary_dict.h
	$(CXX) -std=c++17 -l marisa -o t/01_binary_dict t/01_binary_dict.cc

t/02_skk_dict: t/02_skk_dict.cc src/skkdict.h
	$(CXX) -std=c++17 -l marisa -o t/02_skk_dict t/02_skk_dict.cc

test: t/00_systemlm t/01_binary_dict t/02_skk_dict
	./t/00_systemlm
	./t/01_binary_dict
	./t/02_skk_dict
	pytest tests

# -------------------------------------------------------------------------

install:
	install -m 0755 -d $(DESTDIR)$(DATADIR)/akaza-data
	install -m 0644 akaza_data/data/*.trie $(DESTDIR)$(DATADIR)/akaza-data

.PHONY: all test install
