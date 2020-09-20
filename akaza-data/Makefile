all: akaza_data/data/system_dict.trie akaza_data/data/system_language_model.trie

jawiki-latest-pages-articles.xml.bz2:
	wget --no-verbose --no-clobber -O jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

jawiki-latest-pages-articles.xml: jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep jawiki-latest-pages-articles.xml.bz2

text/BE/wiki_13: jawiki-latest-pages-articles.xml
	python -m wikiextractor.WikiExtractor jawiki-latest-pages-articles.xml

dat/BE/wiki_13: text/BE/wiki_13 bin/wiki2text-runner.py bin/wiki2text.py
	python bin/wiki2text-runner.py

jawiki.wfreq: dat/BE/wiki_13 bin/text2wfreq.py
	python bin/text2wfreq.py

# 118,334=entry count of libkkc-data.
jawiki.vocab: jawiki.wfreq
	python bin/wfreq2vocab.py

jawiki.1gram.json: jawiki.vocab  bin/dumpngram.py
	python bin/dumpngram.py jawiki.vocab

akaza_data/data/system_language_model.trie: jawiki.1gram.json jawiki.2gram.json jawiki.vocab bin/create-system_language_model-from-json.py
	python bin/create-system_language_model-from-json.py

akaza_data/data/system_dict.trie: jawiki.vocab
	python bin/make-system-dict.py

.PHONY: all
