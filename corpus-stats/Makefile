CIRRUS_DATE ?= 20251229
TOKENIZER_OPTS ?=
CC100_LIMIT ?= 5000000
CC100_WEIGHT ?= 0.3

# ローカルビルドの akaza-data を優先的に使う
export PATH := $(CURDIR)/../target/release:$(PATH)

all: work/stats-vibrato-bigram.wordcnt.trie work/stats-vibrato-skip-bigram.wordcnt.trie work/vibrato-ipadic.vocab

all-full: work/stats-vibrato-bigram-full.wordcnt.trie work/stats-vibrato-skip-bigram-full.wordcnt.trie work/vibrato-ipadic-full.vocab

# =========================================================================
#  dist: 配布用 tarball の作成 (jawiki + 青空文庫のみ)
# =========================================================================

dist: all
	mkdir -p dist/
	cp work/stats-vibrato-unigram.wordcnt.trie dist/
	cp work/stats-vibrato-bigram.wordcnt.trie dist/
	cp work/stats-vibrato-skip-bigram.wordcnt.trie dist/
	cp work/vibrato-ipadic.vocab dist/
	cp NOTICE dist/

# =========================================================================
#  dist-full: CC-100 込みの配布用 tarball の作成
# =========================================================================

dist-full: all-full
	mkdir -p dist-full/
	cp work/stats-vibrato-unigram-full.wordcnt.trie dist-full/stats-vibrato-unigram.wordcnt.trie
	cp work/stats-vibrato-bigram-full.wordcnt.trie dist-full/stats-vibrato-bigram.wordcnt.trie
	cp work/stats-vibrato-skip-bigram-full.wordcnt.trie dist-full/stats-vibrato-skip-bigram.wordcnt.trie
	cp work/vibrato-ipadic-full.vocab dist-full/vibrato-ipadic.vocab
	cp NOTICE dist-full/

# =========================================================================
#  release: CalVer タグ作成 + GitHub Release にアップロード
# =========================================================================

release: dist dist-full
	@VER_PREFIX="v$$(date +%Y.%m%d)"; \
	GIT_PREFIX="corpus-stats/$${VER_PREFIX}"; \
	EXISTING=$$(git tag -l "$${GIT_PREFIX}.*" | sort -t. -k4 -n | tail -1); \
	if [ -z "$$EXISTING" ]; then \
		PATCH=0; \
	else \
		PATCH=$$(( $$(echo "$$EXISTING" | rev | cut -d. -f1 | rev) + 1 )); \
	fi; \
	GIT_TAG="$${GIT_PREFIX}.$${PATCH}"; \
	RELEASE_TAG="$${VER_PREFIX}.$${PATCH}"; \
	echo "Creating git tag $$GIT_TAG, release $$RELEASE_TAG ..."; \
	tar czvf akaza-corpus-stats.tar.gz -C dist . && \
	tar czvf akaza-corpus-stats-full.tar.gz -C dist-full . && \
	git tag "$$GIT_TAG" && \
	git push origin "$$GIT_TAG" && \
	gh release create "$$RELEASE_TAG" akaza-corpus-stats.tar.gz akaza-corpus-stats-full.tar.gz \
		--repo akaza-im/akaza-corpus-stats --title "$$RELEASE_TAG" --generate-notes && \
	rm -f akaza-corpus-stats.tar.gz akaza-corpus-stats-full.tar.gz && \
	echo "Released $$RELEASE_TAG (git tag: $$GIT_TAG)"

# =========================================================================
#  Wikipedia (CirrusSearch ダンプ)
# =========================================================================

work/jawiki/jawiki-cirrussearch-content.json.gz:
	mkdir -p work/jawiki/
	wget --show-progress --no-clobber -O work/jawiki/jawiki-cirrussearch-content.json.gz \
		https://dumps.wikimedia.org/other/cirrussearch/$(CIRRUS_DATE)/jawiki-$(CIRRUS_DATE)-cirrussearch-content.json.gz

work/jawiki/extracted/_SUCCESS: work/jawiki/jawiki-cirrussearch-content.json.gz
	python3 scripts/extract-cirrus.py work/jawiki/jawiki-cirrussearch-content.json.gz work/jawiki/extracted/
	touch work/jawiki/extracted/_SUCCESS

# =========================================================================
#  CC-100 Japanese
# =========================================================================

work/cc100/ja.txt.xz:
	mkdir -p work/cc100/
	wget --show-progress --no-clobber -O work/cc100/ja.txt.xz \
		https://data.statmt.org/cc-100/ja.txt.xz

work/cc100/extracted/_SUCCESS: work/cc100/ja.txt.xz
	python3 scripts/extract-cc100.py --limit=$(CC100_LIMIT) \
		work/cc100/ja.txt.xz work/cc100/extracted/
	touch work/cc100/extracted/_SUCCESS

# =========================================================================
#  Vibrato 辞書
# =========================================================================

work/vibrato/ipadic-mecab-2_7_0.tar.xz:
	mkdir -p work/vibrato/
	wget --show-progress --no-clobber -O work/vibrato/ipadic-mecab-2_7_0.tar.xz \
		https://github.com/daac-tools/vibrato/releases/download/v0.5.0/ipadic-mecab-2_7_0.tar.xz

work/vibrato/ipadic-mecab-2_7_0/system.dic: work/vibrato/ipadic-mecab-2_7_0.tar.xz
	mkdir -p work/vibrato/
	tar -xmJf work/vibrato/ipadic-mecab-2_7_0.tar.xz -C work/vibrato/
	zstd -d work/vibrato/ipadic-mecab-2_7_0/system.dic.zst -o work/vibrato/ipadic-mecab-2_7_0/system.dic

# =========================================================================
#  トーカナイズ
# =========================================================================

work/jawiki/vibrato-ipadic/_SUCCESS: mecab-user-dict.csv work/jawiki/extracted/_SUCCESS work/vibrato/ipadic-mecab-2_7_0/system.dic
	akaza-data tokenize \
		--reader=jawiki \
		--user-dict=mecab-user-dict.csv \
		--system-dict=work/vibrato/ipadic-mecab-2_7_0/system.dic \
		$(TOKENIZER_OPTS) \
		work/jawiki/extracted \
		work/jawiki/vibrato-ipadic/ \
		-vvv

work/aozora_bunko/vibrato-ipadic/_SUCCESS: work/vibrato/ipadic-mecab-2_7_0/system.dic
	akaza-data tokenize \
		--reader=aozora_bunko \
		--user-dict=mecab-user-dict.csv \
		--system-dict=work/vibrato/ipadic-mecab-2_7_0/system.dic \
		aozorabunko_text/cards/ \
		work/aozora_bunko/vibrato-ipadic/ -vv

work/cc100/vibrato-ipadic/_SUCCESS: mecab-user-dict.csv work/cc100/extracted/_SUCCESS work/vibrato/ipadic-mecab-2_7_0/system.dic
	akaza-data tokenize \
		--reader=jawiki \
		--user-dict=mecab-user-dict.csv \
		--system-dict=work/vibrato/ipadic-mecab-2_7_0/system.dic \
		$(TOKENIZER_OPTS) \
		work/cc100/extracted \
		work/cc100/vibrato-ipadic/ \
		-vvv

# =========================================================================
#  統計データ生成 (jawiki + 青空文庫のみ)
# =========================================================================

work/vibrato-ipadic.wfreq: work/jawiki/vibrato-ipadic/_SUCCESS work/aozora_bunko/vibrato-ipadic/_SUCCESS
	akaza-data wfreq \
		--src-dir=work/jawiki/vibrato-ipadic/ \
		--src-dir=work/aozora_bunko/vibrato-ipadic/ \
		work/vibrato-ipadic.wfreq -vvv

work/vibrato-ipadic.vocab: work/vibrato-ipadic.wfreq
	akaza-data vocab --threshold 16 work/vibrato-ipadic.wfreq work/vibrato-ipadic.vocab -vvv

work/stats-vibrato-unigram.wordcnt.trie: work/vibrato-ipadic.wfreq
	akaza-data wordcnt-unigram \
		work/vibrato-ipadic.wfreq \
		work/stats-vibrato-unigram.wordcnt.trie -vvv

work/stats-vibrato-bigram.wordcnt.trie: work/stats-vibrato-unigram.wordcnt.trie work/jawiki/vibrato-ipadic/_SUCCESS work/aozora_bunko/vibrato-ipadic/_SUCCESS
	mkdir -p work/dump/
	akaza-data wordcnt-bigram --threshold=3 \
		--corpus-dirs work/jawiki/vibrato-ipadic/ \
		--corpus-dirs work/aozora_bunko/vibrato-ipadic/ \
		work/stats-vibrato-unigram.wordcnt.trie work/stats-vibrato-bigram.wordcnt.trie -vvv

work/stats-vibrato-skip-bigram.wordcnt.trie: work/stats-vibrato-unigram.wordcnt.trie work/jawiki/vibrato-ipadic/_SUCCESS work/aozora_bunko/vibrato-ipadic/_SUCCESS
	akaza-data wordcnt-skip-bigram --threshold=3 \
		--corpus-dirs work/jawiki/vibrato-ipadic/ \
		--corpus-dirs work/aozora_bunko/vibrato-ipadic/ \
		work/stats-vibrato-unigram.wordcnt.trie work/stats-vibrato-skip-bigram.wordcnt.trie -vvv

# =========================================================================
#  統計データ生成 -full (jawiki + 青空文庫 + CC-100)
# =========================================================================

work/vibrato-ipadic-full.wfreq: work/jawiki/vibrato-ipadic/_SUCCESS work/aozora_bunko/vibrato-ipadic/_SUCCESS work/cc100/vibrato-ipadic/_SUCCESS
	akaza-data wfreq \
		--src-dir=work/jawiki/vibrato-ipadic/ \
		--src-dir=work/aozora_bunko/vibrato-ipadic/ \
		--src-dir=work/cc100/vibrato-ipadic/:$(CC100_WEIGHT) \
		work/vibrato-ipadic-full.wfreq -vvv

work/vibrato-ipadic-full.vocab: work/vibrato-ipadic-full.wfreq
	akaza-data vocab --threshold 16 work/vibrato-ipadic-full.wfreq work/vibrato-ipadic-full.vocab -vvv

work/stats-vibrato-unigram-full.wordcnt.trie: work/vibrato-ipadic-full.wfreq
	akaza-data wordcnt-unigram \
		work/vibrato-ipadic-full.wfreq \
		work/stats-vibrato-unigram-full.wordcnt.trie -vvv

work/stats-vibrato-bigram-full.wordcnt.trie: work/stats-vibrato-unigram-full.wordcnt.trie work/jawiki/vibrato-ipadic/_SUCCESS work/aozora_bunko/vibrato-ipadic/_SUCCESS work/cc100/vibrato-ipadic/_SUCCESS
	mkdir -p work/dump/
	akaza-data wordcnt-bigram --threshold=3 \
		--corpus-dirs work/jawiki/vibrato-ipadic/ \
		--corpus-dirs work/aozora_bunko/vibrato-ipadic/ \
		--corpus-dirs work/cc100/vibrato-ipadic/:$(CC100_WEIGHT) \
		work/stats-vibrato-unigram-full.wordcnt.trie work/stats-vibrato-bigram-full.wordcnt.trie -vvv

work/stats-vibrato-skip-bigram-full.wordcnt.trie: work/stats-vibrato-unigram-full.wordcnt.trie work/jawiki/vibrato-ipadic/_SUCCESS work/aozora_bunko/vibrato-ipadic/_SUCCESS work/cc100/vibrato-ipadic/_SUCCESS
	akaza-data wordcnt-skip-bigram --threshold=3 \
		--corpus-dirs work/jawiki/vibrato-ipadic/ \
		--corpus-dirs work/aozora_bunko/vibrato-ipadic/ \
		--corpus-dirs work/cc100/vibrato-ipadic/:$(CC100_WEIGHT) \
		work/stats-vibrato-unigram-full.wordcnt.trie work/stats-vibrato-skip-bigram-full.wordcnt.trie -vvv

# =========================================================================

clean:
	rm -rf dist/ dist-full/

# akaza-data が生成したファイル (tokenize 以降) を削除。Wikipedia 抽出結果は残す。
clean-tokenized:
	rm -rf work/jawiki/vibrato-ipadic/ work/aozora_bunko/vibrato-ipadic/ work/cc100/vibrato-ipadic/ \
		work/vibrato-ipadic.wfreq work/vibrato-ipadic.vocab \
		work/vibrato-ipadic-full.wfreq work/vibrato-ipadic-full.vocab \
		work/stats-vibrato-unigram.wordcnt.trie work/stats-vibrato-bigram.wordcnt.trie \
		work/stats-vibrato-skip-bigram.wordcnt.trie \
		work/stats-vibrato-unigram-full.wordcnt.trie work/stats-vibrato-bigram-full.wordcnt.trie \
		work/stats-vibrato-skip-bigram-full.wordcnt.trie \
		dist/ dist-full/

.PHONY: all all-full dist dist-full release clean clean-tokenized
