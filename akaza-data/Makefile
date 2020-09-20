all: akaza_data/data/system_dict.trie akaza_data/data/system_language_model.trie

work/jawiki-latest-pages-articles.xml.bz2:
	mkdir work/
	wget --no-verbose --no-clobber -O work/jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

work/jawiki-latest-pages-articles.xml: work/jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep work/jawiki-latest-pages-articles.xml.bz2

work/extracted/BE/wiki_13: work/jawiki-latest-pages-articles.xml
	python -m wikiextractor.WikiExtractor --processes 8 --out work/extracted/ work/jawiki-latest-pages-articles.xml

work/text/BE/wiki_13: work/extracted/BE/wiki_13 bin/wiki2text-runner.py bin/wiki2text.py
	python bin/wiki2text-runner.py

work/jawiki.wfreq: work/text/BE/wiki_13 bin/text2wfreq.py
	python bin/text2wfreq.py

# 118,334=entry count of libkkc-data.
work/jawiki.vocab: work/jawiki.wfreq
	python bin/wfreq2vocab.py

work/jawiki.1gram.json: work/jawiki.vocab  bin/dumpngram.py
	python bin/dumpngram.py work/jawiki.vocab

akaza_data/data/system_language_model.trie: work/jawiki.1gram.json work/jawiki.2gram.json work/jawiki.vocab bin/create-system_language_model-from-json.py
	python bin/create-system_language_model-from-json.py

akaza_data/data/system_dict.trie: work/jawiki.vocab
	python bin/make-system-dict.py

.PHONY: all
