test-data: work/vibrato/ipadic-mecab-2_7_0/system.dic

work/vibrato/ipadic-mecab-2_7_0.tar.gz:
	mkdir -p work/vibrato/
	wget --no-verbose --no-clobber -O work/vibrato/ipadic-mecab-2_7_0.tar.gz https://github.com/daac-tools/vibrato/releases/download/v0.3.1/ipadic-mecab-2_7_0.tar.gz

work/vibrato/ipadic-mecab-2_7_0/system.dic: work/vibrato/ipadic-mecab-2_7_0.tar.gz
	mkdir -p work/vibrato/
	tar -xmzf work/vibrato/ipadic-mecab-2_7_0.tar.gz -C work/vibrato/

evaluate:
	AKAZA_ROMKAN_DIR=../romkan/ \
	cargo run --release -- evaluate \
		 --corpus=anthy-corpus/corpus.0.txt \
		 --corpus=anthy-corpus/corpus.1.txt \
		 --corpus=anthy-corpus/corpus.2.txt \
		 --corpus=anthy-corpus/corpus.3.txt \
		 --corpus=anthy-corpus/corpus.4.txt \
		 --corpus=anthy-corpus/corpus.5.txt \
		 --eucjp-dict=/usr/share/skk/SKK-JISYO.L \
		 -v

.PHONY: test-data evaluate

