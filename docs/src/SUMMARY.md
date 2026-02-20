# Summary

[はじめに](introduction.md)

# ユーザーズマニュアル

- [ユーザーズマニュアル](user-manual/README.md)

# 内部構造

- [内部構造の概要](internals/README.md)
  - [データフロー](internals/data-flow.md)
  - [変換エンジンの仕組み](internals/conversion-engine.md)
  - [コーパス学習](internals/learn-corpus.md)
  - [評価方法](internals/evaluation.md)
  - [文節伸縮の仕様](internals/clause-extension-behavior.md)
  - [モデルファイルのメタデータ](internals/model-metadata.md)
  - [ユーザーデータ](internals/user-data.md)
  - [設計メモ・レポート](internals/notes/README.md)
    - [K-Best セグメンテーション](internals/notes/k-best-segmentation.md)
    - [リランキング](internals/notes/reranking.md)
    - [リランキング評価レポート](internals/notes/reranking-evaluation-report.md)
    - [数値+助数詞変換の再設計](internals/notes/numeric-counter-redesign.md)
    - [構造化パーセプトロン](internals/notes/structured-perceptron.md)
    - [learn-corpus 改善実験](internals/notes/learn-corpus-improvement.md)
    - [構造化パーセプトロン評価レポート](internals/notes/structured-perceptron-evaluation.md)
    - [前処理]()
      - [漢方薬の読み問題](internals/notes/preproc/kanpoyaku.md)
      - [MeCab 形態素解析](internals/notes/preproc/mecab.md)
    - [コーパス統計]()
      - [利用可能な日本語コーパス調査](internals/notes/corpus-stats/available-japanese-corpora.md)
      - [CC-100 クリーニング戦略](internals/notes/corpus-stats/cc100-cleaning-strategy.md)
      - [CirrusSearch 日本語コーパス調査](internals/notes/corpus-stats/cirrus-japanese-corpora.md)
      - [jawiktionary 評価](internals/notes/corpus-stats/jawiktionary-evaluation.md)
      - [v2026.0216.0 比較レポート](internals/notes/corpus-stats/v2026.0216.0-comparison.md)
      - [小文字かな辞書エントリの混入問題](internals/notes/corpus-stats/small-kana-dict-contamination.md)
    - [デフォルトモデル]()
      - [変換精度改善の方針](internals/notes/default-model/conversion-improvement-strategy.md)
      - [CC-100 重み付き統合レポート](internals/notes/default-model/cc100-weighted-integration.md)
      - [`<NUM>` トークン正規化レポート](internals/notes/default-model/num-token-normalization.md)
      - [失敗記録: bigram バックオフ補間](internals/notes/default-model/failed-bigram-backoff-interpolation.md)
      - [BAD 傾向分析 (2026-02-16)](internals/notes/default-model/bad-trend-analysis-2026-0216.md)
