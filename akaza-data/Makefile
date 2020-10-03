CXX=clang++
# Note: `shell` is a gnu make extension.
EXT_SUFFIX:=$(shell python3-config --extension-suffix)

all: akaza_data/data/system_dict.trie akaza_data/data/lm_v2_2gram.trie akaza_data/data/lm_v2_2gram.trie binary
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

akaza_data/data/system_dict.trie: work/jawiki.vocab jawiki-kana-kanji-dict/SKK-JISYO.jawiki
	python bin/070_make-system-dict.py
	python bin/080_system_dump.py

bin/090_effective_lm: bin/090_effective_lm.cc
	$(CXX) -l marisa bin/090_effective_lm.cc -o bin/090_effective_lm

akaza_data/data/lm_v2_2gram.trie: bin/090_effective_lm work/jawiki.merged-2gram.txt
	./bin/090_effective_lm

# -------------------------------------------------------------------------

akaza_data/systemlm_loader$(EXT_SUFFIX): akaza_data/systemlm_loader.cc src/system_lm.h
	$(CXX) -O3 -Wall -shared -lmarisa -std=c++11 -fPIC `python3 -m pybind11 --includes` akaza_data/systemlm_loader.cc \
		-o akaza_data/systemlm_loader$(EXT_SUFFIX)

# -------------------------------------------------------------------------

t/00_systemlm: t/00_systemlm.cc
	$(CXX) -std=c++17 -l marisa -o t/00_systemlm t/00_systemlm.cc

test: t/00_systemlm
	./t/00_systemlm
	pytest tests

.PHONY: all test
