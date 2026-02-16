PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share
DESTDIR ?=
MODELDIR ?= $(DESTDIR)$(DATADIR)/akaza/model/default/

CORPUS_STATS_VERSION ?= v2026.0211.1

# ローカルビルドの akaza-data を優先的に使う
export PATH := $(CURDIR)/../target/release:$(PATH)

all: data/bigram.model \
	 data/skip_bigram.model \
	 data/SKK-JISYO.akaza

# -------------------------------------------------------------------------
# corpus-stats tarball ダウンロード
# -------------------------------------------------------------------------

work/corpus-stats/_SUCCESS:
	mkdir -p work/
	gh release download $(CORPUS_STATS_VERSION) \
		--repo akaza-im/akaza-corpus-stats \
		--pattern 'akaza-corpus-stats-full.tar.gz' \
		--output work/akaza-corpus-stats.tar.gz
	tar xzf work/akaza-corpus-stats.tar.gz -C work/
	rm work/akaza-corpus-stats.tar.gz
	mkdir -p work/corpus-stats && touch work/corpus-stats/_SUCCESS

# -------------------------------------------------------------------------
#  Unidic の処理
# -------------------------------------------------------------------------

work/unidic/unidic.zip:
	mkdir -p work/unidic/
	gh release download unidic-mirror \
		--repo akaza-im/akaza \
		--pattern 'unidic-csj-3.1.1.zip' \
		--output work/unidic/unidic.zip

work/unidic/lex_3_1.csv: work/unidic/unidic.zip
	unzip -D -o -j work/unidic/unidic.zip -d work/unidic/
	touch work/unidic/lex_3_1.csv

# -------------------------------------------------------------------------

# 統計的仮名かな漢字変換のためのモデル作成処理

data/bigram.model: work/corpus-stats/_SUCCESS training-corpus/must.txt training-corpus/should.txt training-corpus/may.txt data/SKK-JISYO.akaza
	mkdir -p data
	akaza-data learn-corpus \
		--delta=2000 \
		--may-epochs=10 \
		--should-epochs=100 \
		--must-epochs=10000 \
		training-corpus/may.txt \
		training-corpus/should.txt \
		training-corpus/must.txt \
		work/stats-vibrato-unigram.wordcnt.trie work/stats-vibrato-bigram.wordcnt.trie \
		data/unigram.model data/bigram.model \
		--src-skip-bigram=work/stats-vibrato-skip-bigram.wordcnt.trie \
		--dst-skip-bigram=data/skip_bigram.model \
		-v

data/unigram.model: data/bigram.model

data/skip_bigram.model: data/bigram.model

# -------------------------------------------------------------------------

# システム辞書の構築。dict/SKK-JISYO.akaza、コーパスに書かれている語彙および work/vibrato-ipadic.vocab にある語彙。
# から、SKK-JISYO.L に含まれる語彙を除いたものが登録されている。

data/SKK-JISYO.akaza: work/corpus-stats/_SUCCESS dict/SKK-JISYO.akaza training-corpus/must.txt training-corpus/should.txt training-corpus/may.txt work/unidic/lex_3_1.csv
	mkdir -p data
	akaza-data make-dict \
		--corpus training-corpus/must.txt \
		--corpus training-corpus/should.txt \
		--corpus training-corpus/may.txt \
		--unidic work/unidic/lex_3_1.csv \
		--vocab work/vibrato-ipadic.vocab \
		data/SKK-JISYO.akaza \
		-vvv

# -------------------------------------------------------------------------

# corpus.4.txt は誤変換をおさめたものなので評価用には使わない
evaluate: data/bigram.model
	CORPUS_STATS_VERSION=$(CORPUS_STATS_VERSION) scripts/run-evaluate.sh

# -------------------------------------------------------------------------

install:
	install -m 0755 -d $(MODELDIR)
	install -m 0644 data/*.model $(MODELDIR)
	install -m 0644 data/SKK-JISYO.* $(MODELDIR)

# -------------------------------------------------------------------------

.PHONY: all install evaluate
