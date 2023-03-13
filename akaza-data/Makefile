test-data: work/vibrato/ipadic-mecab-2_7_0/system.dic

work/vibrato/ipadic-mecab-2_7_0.tar.xz:
	mkdir -p work/vibrato/
	wget --no-verbose --no-clobber -O work/vibrato/ipadic-mecab-2_7_0.tar.xz https://github.com/daac-tools/vibrato/releases/download/v0.5.0/ipadic-mecab-2_7_0.tar.xz

work/vibrato/ipadic-mecab-2_7_0/system.dic: work/vibrato/ipadic-mecab-2_7_0.tar.xz
	mkdir -p work/vibrato/
	tar -xmJf work/vibrato/ipadic-mecab-2_7_0.tar.xz -C work/vibrato/

evaluate:
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

