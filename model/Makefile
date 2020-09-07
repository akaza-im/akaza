all: jawiki.1gram

jawiki-latest-pages-articles.xml.bz2:
	wget --no-verbose --no-clobber -O jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

jawiki-latest-pages-articles.xml: jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep jawiki-latest-pages-articles.xml.bz2

text/BE/wiki_13: jawiki-latest-pages-articles.xml
	python -m wikiextractor.WikiExtractor jawiki-latest-pages-articles.xml

dat/BE/wiki_13: text/BE/wiki_13 bin/wiki2text.py bin/wiki2text-runner.py
	python bin/wiki2text-runner.py

jawiki.text: dat/BE/wiki_13
	cat dat/*/wiki_* > jawiki.text

jawiki.wfreq: jawiki.text
	text2wfreq jawiki.text jawiki.wfreq

# 118,334=entry count of libkkc-data.
jawiki.vocab: jawiki.wfreq
	wfreq2vocab -top 30000 jawiki.wfreq jawiki.vocab

jawiki.id3gram: jawiki.vocab
	text2idngram -n 2 -temp /tmp/ -vocab jawiki.vocab jawiki.text jawiki.id3gram

jawiki.arpa: jawiki.id3gram jawiki.vocab
	idngram2lm -idngram jawiki.id3gram -vocab jawiki.vocab -arpa jawiki.arpa

jawiki.1gram: bin/naive.py jawiki.arpa
	python bin/naive.py jawiki.arpa

.PHONY: all

