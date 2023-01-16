PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share
DESTDIR ?=


all: data/bigram.model \
	 data/bigram.model \
	 data/SKK-JISYO.akaza

# -------------------------------------------------------------------------

# wikipedia の前処理

work/jawiki/jawiki-latest-pages-articles.xml.bz2:
	mkdir -p work/jawiki/
	wget --no-verbose --no-clobber -O work/jawiki/jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

work/jawiki/jawiki-latest-pages-articles.xml: work/jawiki/jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep work/jawiki/jawiki-latest-pages-articles.xml.bz2

work/jawiki/extracted/_SUCCESS: work/jawiki/jawiki-latest-pages-articles.xml
	python3 -m wikiextractor.WikiExtractor --quiet --processes 8 --out work/jawiki/extracted/ work/jawiki/jawiki-latest-pages-articles.xml
	touch work/jawiki/extracted/_SUCCESS

# -------------------------------------------------------------------------

# Vibrato トーカナイズ

work/vibrato/ipadic-mecab-2_7_0.tar.gz:
	mkdir -p work/vibrato/
	wget --no-verbose --no-clobber -O work/vibrato/ipadic-mecab-2_7_0.tar.gz https://github.com/daac-tools/vibrato/releases/download/v0.3.1/ipadic-mecab-2_7_0.tar.gz

work/vibrato/ipadic-mecab-2_7_0/system.dic: work/vibrato/ipadic-mecab-2_7_0.tar.gz
	mkdir -p work/vibrato/
	tar -xmzf work/vibrato/ipadic-mecab-2_7_0.tar.gz -C work/vibrato/

work/jawiki/vibrato-ipadic/_SUCCESS: src/subcmd/tokenize.rs mecab-user-dict.csv src/corpus_reader/wikipedia_extracted.rs work/jawiki/extracted/_SUCCESS work/vibrato/ipadic-mecab-2_7_0/system.dic
	cargo run --release -- tokenize-vibrato-ipadic --user-dict=mecab-user-dict.csv work/vibrato/ipadic-mecab-2_7_0/system.dic work/jawiki/extracted work/jawiki/vibrato-ipadic/ -vvv

work/aozora_bunko/vibrato-ipadic/_SUCCESS: src/corpus_reader/aozora_bunko.rs work/vibrato/ipadic-mecab-2_7_0/system.dic
	cargo run --release -- tokenize-aozora-bunko-vibrato-ipadic work/vibrato/ipadic-mecab-2_7_0/system.dic aozorabunko_text/cards/ work/aozora_bunko/vibrato-ipadic/ -vv

work/vibrato-ipadic.wfreq: work/jawiki/vibrato-ipadic/_SUCCESS src/subcmd/wfreq.rs work/aozora_bunko/vibrato-ipadic/_SUCCESS
	cargo run --release -- wfreq work/jawiki/vibrato-ipadic/ work/aozora_bunko/vibrato-ipadic/ corpus/ work/vibrato-ipadic.wfreq -vvv

# threshold が 16 なのはヒューリスティックなパラメータ設定による。
# vocab ファイルを作る意味は、辞書の作成のためだけなので、わざわざ作らなくてもよいかもしれない。
work/vibrato-ipadic.vocab: work/vibrato-ipadic.wfreq src/subcmd/vocab.rs
	cargo run --release -- vocab --threshold 16 work/vibrato-ipadic.wfreq work/vibrato-ipadic.vocab -vvv


# -------------------------------------------------------------------------

# 統計的仮名かな漢字変換のためのモデル作成処理

work/stats-vibrato-unigram.raw.trie: work/vibrato-ipadic.wfreq
	cargo run --release -- make-stats-system-unigram-lm work/vibrato-ipadic.wfreq work/stats-vibrato-unigram.raw.trie

work/stats-vibrato-bigram.raw.trie: work/stats-vibrato-unigram.raw.trie work/stats-vibrato-unigram.raw.trie src/subcmd/make_stats_system_bigram_lm.rs work/aozora_bunko/vibrato-ipadic/_SUCCESS
	cargo run --release -- make-stats-system-bigram-lm --threshold=3 \
		--corpus-dirs work/jawiki/vibrato-ipadic/ \
		--corpus-dirs work/aozora_bunko/vibrato-ipadic/ \
		work/stats-vibrato-unigram.raw.trie work/stats-vibrato-bigram.raw.trie

data/bigram.model: work/stats-vibrato-bigram.raw.trie work/stats-vibrato-unigram.raw.trie src/subcmd/learn_corpus.rs corpus/must.txt corpus/should.txt corpus/may.txt data/SKK-JISYO.akaza
	cargo run --release -- learn-corpus \
		--delta=0.5 \
		--may-epochs=10 \
		--should-epochs=100 \
		--must-epochs=10000 \
		corpus/may.txt \
		corpus/should.txt \
		corpus/must.txt \
		work/stats-vibrato-unigram.raw.trie work/stats-vibrato-bigram.raw.trie \
		data/unigram.model data/bigram.model \
		-v

data/unigram.model: data/bigram.model

# -------------------------------------------------------------------------

# システム辞書の構築。dict/SKK-JISYO.akaza、コーパスに書かれている語彙および work/vibrato-ipadic.vocab にある語彙。
# から、SKK-JISYO.L に含まれる語彙を除いたものが登録されている。

data/SKK-JISYO.akaza: work/vibrato-ipadic.vocab dict/SKK-JISYO.akaza src/subcmd/make_dict.rs  corpus/must.txt corpus/should.txt corpus/may.txt
	cargo run --release -- make-system-dict \
		--corpus corpus/must.txt \
		--corpus corpus/should.txt \
		--corpus corpus/may.txt \
		work/vibrato-ipadic.vocab \
		data/SKK-JISYO.akaza \
		-vvv

# -------------------------------------------------------------------------

evaluate:
	AKAZA_DATA_DIR=data/ cargo run --release evaluate anthy-corpus -v

# -------------------------------------------------------------------------

install:
	install -m 0755 -d $(DESTDIR)$(DATADIR)/akaza-data
	install -m 0644 data/*.trie $(DESTDIR)$(DATADIR)/akaza-data

# -------------------------------------------------------------------------

test-data: work/vibrato/ipadic-mecab-2_7_0/system.dic

.PHONY: all install evaluate test-data

