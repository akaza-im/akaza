PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share
DESTDIR ?=


all: data/system_dict.trie data/lm_v2_2gram.trie data/lm_v2_2gram.trie \
	data/system_dict.trie data/single_term.trie

work/jawiki-latest-pages-articles.xml.bz2:
	mkdir -p work/
	wget --no-verbose --no-clobber -O work/jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

work/jawiki-latest-pages-articles.xml: work/jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep work/jawiki-latest-pages-articles.xml.bz2

work/extracted/BE/wiki_13: work/jawiki-latest-pages-articles.xml
	python3 -m wikiextractor.WikiExtractor --quiet --processes 8 --out work/extracted/ work/jawiki-latest-pages-articles.xml

work/annotated/BE/wiki_13: work/extracted/BE/wiki_13 bin/010_wiki2annotated.py
	python3 bin/010_wiki2annotated.py

work/text/BE/wiki_13: work/annotated/BE/wiki_13 bin/020_annotated2text.py bin/akaza_data_utils/merge_terms.py
	python3 bin/020_annotated2text.py

work/jawiki.wfreq: work/text/BE/wiki_13 bin/030_text2wfreq.py
	python3 bin/030_text2wfreq.py

# 118,334=entry count of libkkc-data.
work/jawiki.vocab: work/jawiki.wfreq bin/040_wfreq2vocab.py
	python3 bin/040_wfreq2vocab.py

# very slow
work/ngram/BE/wiki_13.1gram.txt: work/jawiki.vocab  bin/050_dumpngram.py
	python3 bin/050_dumpngram.py work/jawiki.vocab

work/jawiki.merged-2gram.txt: work/ngram/BE/wiki_13.1gram.txt work/jawiki.vocab bin/060_create-system_language_model.py
	python3 bin/060_create-system_language_model.py

# システム辞書の構築
work/jawiki.single_term.txt: work/jawiki.vocab jawiki-kana-kanji-dict/SKK-JISYO.jawiki dict/SKK-JISYO.akaza src/subcmd/make_text_dict.rs
	cargo run --release -- make-text-dict -vvv

data/lm_v2_2gram.trie: work/jawiki.merged-2gram.txt
	cargo run -- make-system-language-model work/jawiki.merged-1gram.txt data/lm_v2_1gram.trie \
    	work/jawiki.merged-2gram.txt data/lm_v2_2gram.trie

data/system_dict.trie: work/jawiki.single_term.txt
	cargo run -- make-system-dict work/jawiki.system_dict.txt data/system_dict.trie

data/single_term.trie: work/jawiki.single_term.txt
	cargo run -- make-system-dict work/jawiki.single_term.txt data/single_term.trie

# -------------------------------------------------------------------------

# http://www.phontron.com/kytea/train-ja.html
# モでルをいじってつかおうとこころみたが、うまくいってないのでいったん、つかわないことにする。

work/kytea/kytea-0.4.2.feat.gz:
	mkdir -p work/kytea/
	wget http://www.phontron.com/kytea/download/kytea-0.4.2.feat.gz -O work/kytea/kytea-0.4.2.feat.gz

work/kytea/kytea-0.4.2.feat: work/kytea/kytea-0.4.2.feat.gz
	gunzip --keep work/kytea/kytea-0.4.2.feat.gz

work/kytea/train.mod: work/kytea/kytea-0.4.2.feat corpus/kytea.txt
	train-kytea -feat work/kytea/kytea-0.4.2.feat -full corpus/kytea.txt -model work/kytea/train.mod

# -------------------------------------------------------------------------

evaluate:
	cargo run --release evaluate anthy-corpus data -v

# -------------------------------------------------------------------------

install:
	install -m 0755 -d $(DESTDIR)$(DATADIR)/akaza-data
	install -m 0644 data/*.trie $(DESTDIR)$(DATADIR)/akaza-data

.PHONY: all install evaluate
