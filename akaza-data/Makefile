PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share
DESTDIR ?=


all: data/stats-kytea-system_dict.trie data/stats-kytea-lm_v2_2gram.trie data/stats-kytea-lm_v2_2gram.trie \
	data/stats-kytea-system_dict.trie data/stats-kytea-single_term.trie work/jawiki/vibrato-ipadic.vocab

# -------------------------------------------------------------------------

# wikipedia の前処理

work/jawiki/jawiki-latest-pages-articles.xml.bz2:
	mkdir -p work/
	wget --no-verbose --no-clobber -O work/jawiki/jawiki-latest-pages-articles.xml.bz2 https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2

work/jawiki/jawiki-latest-pages-articles.xml: work/jawiki/jawiki-latest-pages-articles.xml.bz2
	bunzip2 --keep work/jawiki/jawiki-latest-pages-articles.xml.bz2

work/jawiki/extracted/_SUCCESS: work/jawiki/jawiki-latest-pages-articles.xml
	python3 -m wikiextractor.WikiExtractor --quiet --processes 8 --out work/jawiki/extracted/ work/jawiki/jawiki-latest-pages-articles.xml
	touch work/jawiki/extracted/_SUCCESS

# -------------------------------------------------------------------------

# Vibrato トーカナイズ

work/vibrato/ipadic-mecab-2_7_0.tar.gz:
	wget --no-verbose --no-clobber -O work/vibrato/ipadic-mecab-2_7_0.tar.gz https://github.com/daac-tools/vibrato/releases/download/v0.3.1/ipadic-mecab-2_7_0.tar.gz

work/vibrato/ipadic-mecab-2_7_0/system.dic: work/vibrato/ipadic-mecab-2_7_0.tar.gz
	tar -xzf work/vibrato/ipadic-mecab-2_7_0.tar.gz -C work/vibrato/

 work/jawiki/vibrato-ipadic/_SUCCESS: src/subcmd/tokenize.rs jawiki-kana-kanji-dict/mecab-userdic.csv src/wikipedia/wikipedia_extracted.rs work/jawiki/extracted/_SUCCESS
	cargo run -- tokenize -t vibrato-ipadic work/jawiki/extracted work/jawiki/vibrato-ipadic/ -vvv

 work/jawiki/vibrato-ipadic.wfreq: work/jawiki/vibrato-ipadic/_SUCCESS src/subcmd/wfreq.rs
	cargo run -- wfreq work/jawiki/vibrato-ipadic/ work/jawiki/vibrato-ipadic.wfreq -vvv

# threshold が 16 なのはヒューリスティックなパラメータ設定による。
# vocab ファイルを作る意味は、辞書の作成のためだけなので、わざわざ作らなく手もよいかもしれない。
 work/jawiki/vibrato-ipadic.vocab: work/jawiki/vibrato-ipadic.wfreq src/subcmd/vocab.rs
	cargo run -- vocab --threshold 16 work/jawiki/vibrato-ipadic.wfreq work/jawiki/vibrato-ipadic.vocab -vvv


# -------------------------------------------------------------------------

# Lindera の処理

 work/jawiki/lindera-ipadic/_SUCCESS: src/subcmd/tokenize.rs jawiki-kana-kanji-dict/mecab-userdic.csv src/wikipedia/wikipedia_extracted.rs work/jawiki/extracted/_SUCCESS
	cargo run -- tokenize -u jawiki-kana-kanji-dict/lindera-userdic.bin -t lindera-ipadic work/jawiki/extracted/ work/jawiki/lindera-ipadic/ -vvv

 work/jawiki/lindera-ipadic.wfreq: work/jawiki/lindera-ipadic/_SUCCESS src/subcmd/wfreq.rs
	cargo run -- wfreq work/jawiki/lindera-ipadic/ work/jawiki/lindera-ipadic.wfreq -vvv

# threshold が 16 なのは、ヒューリスティックなパラメータ設定による。
# 調整の余地あり。
 work/jawiki/lindera-ipadic.vocab: work/jawiki/lindera-ipadic.wfreq src/subcmd/vocab.rs
	cargo run -- vocab --threshold 16 work/jawiki/lindera-ipadic.wfreq work/jawiki/lindera-ipadic.vocab -vvv


# -------------------------------------------------------------------------

# 統計的仮名かな漢字変換のためのモデル作成処理

data/stats-vibrato-unigram.trie: work/jawiki/vibrato-ipadic.wfreq
	cargo run -- make-stats-system-unigram-lm work/jawiki/vibrato-ipadic.wfreq data/stats-vibrato-unigram.trie

# TODO: 未実装
data/stats-vibrato-bigram.trie: data/stats-vibrato-unigram.trie data/stats-vibrato-unigram.trie
	cargo run -- make-stats-system-bigram-lm data/stats-vibrato-unigram.trie work/jawiki/vibrato-ipadic/ data/stats-vibrato-bigram.trie

# -------------------------------------------------------------------------

# システム辞書の構築。現在は SKK の辞書などに加えて work/jawiki/vibrato-ipadic.vocab の語彙を混ぜている。
# さらに、コーパスファイルに書かれている辞書の内容も入れている。
#
# 辞書の元になるvocab ファイルについては、ブレが少ない辞書ベースの形態素解析エンジンを利用した方が良いと思う。
# kytea だと素頓狂な読み方を錬成することがある。

# そして、そもそもテキストファイル生成フェーズとバイナリ生成フェーズにわける意味があまりないかもしれない。

# work/system_dict.txt, work/single_term.txt を生成しているのは、辞書の状況調査のため。

work/single_term.trie: src/subcmd/make_text_dict.rs
	cargo run --release -- make-single-term \
		work/single_term.txt \
		data/single_term.trie \
		-vvv

data/system_dict.trie: work/jawiki/vibrato-ipadic.vocab jawiki-kana-kanji-dict/SKK-JISYO.jawiki dict/SKK-JISYO.akaza src/subcmd/make_text_dict.rs
	cargo run --release -- make-system-dict \
		--vocab-file work/jawiki/vibrato-ipadic.vocab \
		work/system_dict.txt \
		data/system_dict.trie \
		-vvv

# -------------------------------------------------------------------------

evaluate:
	cargo run --release evaluate anthy-corpus data -v

# -------------------------------------------------------------------------

install:
	install -m 0755 -d $(DESTDIR)$(DATADIR)/akaza-data
	install -m 0644 data/*.trie $(DESTDIR)$(DATADIR)/akaza-data

.PHONY: all install evaluate
