PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share
DESTDIR ?=


all: akaza_data/data/system_dict.trie akaza_data/data/lm_v2_2gram.trie akaza_data/data/lm_v2_2gram.trie \
	akaza_data/data/system_dict.trie akaza_data/data/single_term.trie

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

# very slow
work/ngram/BE/wiki_13.1gram.txt: work/jawiki.vocab  bin/050_dumpngram.py
	python bin/050_dumpngram.py work/jawiki.vocab

work/jawiki.merged-2gram.txt: work/ngram/BE/wiki_13.1gram.txt work/jawiki.vocab bin/060_create-system_language_model.py
	python bin/060_create-system_language_model.py

work/jawiki.single_dict.txt: work/jawiki.vocab jawiki-kana-kanji-dict/SKK-JISYO.jawiki dict/SKK-JISYO.akaza
	python bin/070_make-system-dict.py
	python bin/080_system_dump.py

akaza_data/data/lm_v2_2gram.trie: work/jawiki.merged-2gram.txt
	akaza-make-system-lm work/jawiki.merged-1gram.txt akaza_data/data/lm_v2_1gram.trie \
    	work/jawiki.merged-2gram.txt akaza_data/data/lm_v2_2gram.trie

akaza_data/data/system_dict.trie: work/jawiki.single_dict.txt
	akaza-make-binary-dict work/jawiki.system_dict.txt akaza_data/data/system_dict.trie

akaza_data/data/single_term.trie: work/jawiki.single_term.txt
	akaza-make-binary-dict work/jawiki.single_term.txt akaza_data/data/single_term.trie

# -------------------------------------------------------------------------

install:
	install -m 0755 -d $(DESTDIR)$(DATADIR)/akaza-data
	install -m 0644 akaza_data/data/*.trie $(DESTDIR)$(DATADIR)/akaza-data

.PHONY: all install
