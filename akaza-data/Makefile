all: akaza_data/data/system_dict.trie akaza_data/data/system_language_model.trie

work/jawiki-latest-pages-articles.xml.bz2:
	mkdir -p work/
	wget --no-verbose --no-clobber -O work/jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

work/jawiki-latest-pages-articles.xml: work/jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep work/jawiki-latest-pages-articles.xml.bz2

work/extracted/BE/wiki_13: work/jawiki-latest-pages-articles.xml
	python -m wikiextractor.WikiExtractor --quiet --processes 8 --out work/extracted/ work/jawiki-latest-pages-articles.xml

work/annotated/BE/wiki_13: work/extracted/BE/wiki_13 bin/010_wiki2annotated.py
	python bin/010_wiki2annotated.py

work/text/BE/wiki_13: work/annotated/BE/wiki_13 bin/020_annotated2text.py
	python bin/020_annotated2text.py

work/jawiki.wfreq: work/text/BE/wiki_13 bin/030_text2wfreq.py
	python bin/030_text2wfreq.py

# 118,334=entry count of libkkc-data.
work/jawiki.vocab: work/jawiki.wfreq bin/040_wfreq2vocab.py
	python bin/040_wfreq2vocab.py

work/jawiki.1gram.json: work/jawiki.vocab  bin/050_dumpngram.py
	python bin/050_dumpngram.py work/jawiki.vocab

akaza_data/data/system_language_model.trie: work/jawiki.1gram.json work/jawiki.2gram.json work/jawiki.vocab bin/060_create-system_language_model.py
	python bin/060_create-system_language_model.py

akaza_data/data/system_dict.trie: work/jawiki.vocab jawiki-kana-kanji-dict/SKK-JISYO.jawiki
	python bin/070_make-system-dict.py

.PHONY: all
