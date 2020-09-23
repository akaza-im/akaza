all: akaza_data/data/system_dict.trie akaza_data/data/system_language_model.trie

work/jawiki-latest-pages-articles.xml.bz2:
	mkdir work/
	wget --no-verbose --no-clobber -O work/jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

work/jawiki-latest-pages-articles.xml: work/jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep work/jawiki-latest-pages-articles.xml.bz2

work/extracted/BE/wiki_13: work/jawiki-latest-pages-articles.xml
	python -m wikiextractor.WikiExtractor --processes 8 --out work/extracted/ work/jawiki-latest-pages-articles.xml

work/text/BE/wiki_13: work/extracted/BE/wiki_13 bin/010_wiki2text-runner.py bin/011_wiki2text.py
	python bin/010_wiki2text-runner.py

work/jawiki.wfreq: work/text/BE/wiki_13 bin/020_text2wfreq.py
	python bin/020_text2wfreq.py

# 118,334=entry count of libkkc-data.
work/jawiki.vocab: work/jawiki.wfreq bin/030_wfreq2vocab.py
	python bin/030_wfreq2vocab.py

work/2gram/BE/wiki_13.2gram.json: work/jawiki.vocab bin/040_text2ngram.py
	python bin/040_text2ngram.py work/jawiki.vocab

work/jawiki.2gram-merged.json: work/jawiki.vocab bin/050_merge2gram.py
	python bin/050_merge2gram.py

work/jawiki.2gram.json: work/jawiki.2gram-merged.json bin/060_ngram-cutoff.py
	python bin/060_ngram-cutoff.py --bigram-cutoff=3

akaza_data/data/system_language_model.trie: work/jawiki.2gram.json work/jawiki.vocab bin/070_create-system_language_model-from-json.py
	python bin/070_create-system_language_model-from-json.py

akaza_data/data/system_dict.trie: work/jawiki.vocab dict/SKK-JISYO.akaza
	python bin/080_make-system-dict.py

.PHONY: all
