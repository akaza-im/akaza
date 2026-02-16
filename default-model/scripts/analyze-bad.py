#!/usr/bin/env python3
"""BAD エントリの傾向分析を行い、カテゴリ別内訳・頻出パターンを出力する。

Usage:
    python3 scripts/analyze-bad.py [evaluate_dir]
    python3 scripts/analyze-bad.py  # 引数なしで最新を使用

出力:
    1. カテゴリ別内訳 (表記揺れ・同音異義語・数詞関連 等)
    2. 高頻度の1文字差 同音異義語ペア
    3. 分節崩壊・未変換・過変換のサブカテゴリ分析
    4. accept.tsv に追加可能な表記揺れの候補数
"""

import os
import re
import sys
from collections import Counter, defaultdict


# ============================================================
# 表記揺れ検出用のペア
# extract-patterns.py の SKIP_PATTERNS と同じ基準
# ============================================================
STYLE_PAIRS = [
    ("もの", "物"), ("こと", "事"), ("ない", "無い"), ("ある", "有る"),
    ("いい", "良い"), ("よい", "良い"), ("いう", "言う"), ("できる", "出来る"),
    ("ところ", "所"), ("ため", "為"), ("ほど", "程"), ("わけ", "訳"),
    ("とき", "時"), ("うれしい", "嬉しい"), ("おいしい", "美味しい"),
    ("かわいい", "可愛い"), ("すべて", "全て"), ("たぶん", "多分"),
    ("すでに", "既に"), ("ほとんど", "殆ど"), ("さっそく", "早速"),
    ("および", "及び"), ("ください", "下さい"), ("いただ", "頂"),
    ("もらう", "貰う"), ("もらっ", "貰っ"), ("いろいろ", "色々"),
    ("ダメ", "だめ"), ("アホ", "あほ"), ("バカ", "馬鹿"), ("マジ", "まじ"),
    ("マシ", "まし"), ("うまく", "上手く"), ("うまい", "上手い"),
    ("ありえ", "有り得"), ("あさって", "明後日"), ("かまわ", "構わ"),
    ("イマイチ", "いまいち"), ("たたん", "畳ん"),
    ("ウーロン茶", "烏龍茶"), ("エビ", "海老"),
    ("スッキリ", "すっきり"), ("ダラダラ", "だらだら"),
    ("ポカン", "ぽかん"), ("おどかし", "脅かし"), ("ねて", "寝て"),
    ("なぜ", "何故"), ("ゴミ", "ごみ"), ("キレイ", "きれい"),
    ("わり", "割"), ("まった", "全"), ("分", "わ"),
    ("付", "つ"), ("行", "い"), ("鳴", "な"), ("見", "み"),
    ("来", "き"), ("後", "あと"), ("間", "あいだ"), ("他", "ほか"),
    ("何", "なん"), ("何", "なに"), ("通", "とお"), ("辺", "あた"),
    ("出来", "でき"), ("上手", "うま"), ("一人", "ひとり"),
    ("二人", "ふたり"), ("頃", "ころ"), ("毎", "ごと"),
    ("渡", "わた"), ("続", "つづ"), ("がんば", "頑張"),
    ("しゃべ", "喋"), ("台詞", "セリフ"), ("マンガ", "漫画"),
    ("ダメ", "駄目"), ("?", "？"), ("!", "！"), ("…", "。。。"),
    (".", "。"), ("%", "％"),
]


def find_latest_evaluate_dir(base_dir="tmp/evaluate"):
    """最新の evaluate ディレクトリを探す。"""
    dirs = []
    for d in os.listdir(base_dir):
        full = os.path.join(base_dir, d)
        if os.path.isdir(full) and d.startswith("2"):
            dirs.append(full)
    if not dirs:
        print("ERROR: evaluate ディレクトリが見つかりません", file=sys.stderr)
        sys.exit(1)
    return sorted(dirs)[-1]


def parse_bad_line(line):
    """[BAD] 行をパースして (reading, corpus, akaza) を返す。"""
    m = re.match(r"\[BAD\]\s+(.+?)\s+=>\s+corpus=(.+?),\s+akaza=(.+)", line.strip())
    if not m:
        return None
    return m.group(1), m.group(2), m.group(3)


def is_style_diff(corpus, akaza):
    """corpus と akaza の差が表記揺れのみかどうかを判定する。"""
    diff_c = corpus
    diff_a = akaza
    for h, k in STYLE_PAIRS:
        diff_c = diff_c.replace(h, "\x00").replace(k, "\x00")
        diff_a = diff_a.replace(h, "\x00").replace(k, "\x00")
    return diff_c == diff_a


def is_punctuation_diff(corpus, akaza):
    """全角半角・句読点の差のみかどうかを判定する。"""
    c_norm = corpus.replace("?", "？").replace("!", "！").replace(".", "。").replace("%", "％")
    return c_norm == akaza


def categorize(entries):
    """エントリをカテゴリに分類する。"""
    categories = defaultdict(list)
    for reading, corpus, akaza in entries:
        # 数詞→アラビア数字化
        if (re.search(r"\d+", akaza) and not re.search(r"\d+", corpus)
                and not re.search(r"\d+", reading)):
            categories["数詞→アラビア数字化"].append((reading, corpus, akaza))
            continue

        # 全角半角・句読点差
        if is_punctuation_diff(corpus, akaza):
            categories["全角半角・句読点差"].append((reading, corpus, akaza))
            continue

        # 表記揺れ
        if is_style_diff(corpus, akaza):
            categories["表記揺れ（ひらがな⇔漢字・カタカナ）"].append((reading, corpus, akaza))
            continue

        # 数字含み文の誤変換
        if re.search(r"\d+", reading) or re.search(r"\d+", corpus):
            categories["数字含み文の誤変換"].append((reading, corpus, akaza))
            continue

        # それ以外
        categories["同音異義語・その他"].append((reading, corpus, akaza))

    return categories


def analyze_single_char_pairs(entries):
    """1文字差の同音異義語ペアを集計する。"""
    pair_examples = defaultdict(list)
    for reading, corpus, akaza in entries:
        if len(corpus) != len(akaza):
            continue
        diffs = [(corpus[i], akaza[i]) for i in range(len(corpus)) if corpus[i] != akaza[i]]
        if len(diffs) == 1:
            c, a = diffs[0]
            pair_examples[(c, a)].append((reading, corpus, akaza))
    return pair_examples


def analyze_diff_patterns(entries):
    """corpus/akaza の差分文字列を集計する。"""
    diff_patterns = Counter()
    for reading, corpus, akaza in entries:
        i = 0
        while i < len(corpus) and i < len(akaza) and corpus[i] == akaza[i]:
            i += 1
        j_c = len(corpus) - 1
        j_a = len(akaza) - 1
        while j_c > i and j_a > i and corpus[j_c] == akaza[j_a]:
            j_c -= 1
            j_a -= 1
        c_diff = corpus[i:j_c + 1]
        a_diff = akaza[i:j_a + 1]
        if c_diff and a_diff:
            diff_patterns[(c_diff, a_diff)] += 1
    return diff_patterns


def analyze_subcategories(entries):
    """同音異義語をサブカテゴリに分類する。"""
    sub_cats = defaultdict(list)
    for reading, corpus, akaza in entries:
        akaza_hira = sum(1 for c in akaza if "\u3040" <= c <= "\u309f")
        corpus_hira = sum(1 for c in corpus if "\u3040" <= c <= "\u309f")

        # 長さが大きく異なる → 分節崩壊
        if abs(len(corpus) - len(akaza)) >= 3:
            sub_cats["分節崩壊"].append((reading, corpus, akaza))
        elif akaza_hira > corpus_hira + 3:
            sub_cats["未変換（ひらがな残り）"].append((reading, corpus, akaza))
        elif akaza_hira < corpus_hira - 3:
            sub_cats["過変換（漢字化）"].append((reading, corpus, akaza))
        else:
            sub_cats["同音漢字置換"].append((reading, corpus, akaza))

    return sub_cats


def count_accept_candidates(entries):
    """accept.tsv に追加可能な表記揺れの候補数を集計する。"""
    candidates = Counter()
    for reading, corpus, akaza in entries:
        if len(corpus) != len(akaza):
            continue
        diff_types = set()
        is_candidate = True
        for i in range(len(corpus)):
            if corpus[i] == akaza[i]:
                continue
            c_hira = "\u3040" <= corpus[i] <= "\u309f"
            c_kata = "\u30a0" <= corpus[i] <= "\u30ff"
            c_kanji = "\u4e00" <= corpus[i] <= "\u9fff"
            a_hira = "\u3040" <= akaza[i] <= "\u309f"
            a_kata = "\u30a0" <= akaza[i] <= "\u30ff"
            a_kanji = "\u4e00" <= akaza[i] <= "\u9fff"
            if (c_hira or c_kata) and (a_hira or a_kata):
                diff_types.add("hira_kata")
            elif (c_hira and a_kanji) or (c_kanji and a_hira):
                diff_types.add("hira_kanji")
            elif (c_kata and a_kanji) or (c_kanji and a_kata):
                diff_types.add("kata_kanji")
            else:
                is_candidate = False
                break
        if is_candidate and diff_types:
            candidates[tuple(sorted(diff_types))] += 1
    return candidates


def main():
    if len(sys.argv) >= 2:
        eval_dir = sys.argv[1]
    else:
        eval_dir = find_latest_evaluate_dir()

    bad_file = os.path.join(eval_dir, "bad.txt")
    if not os.path.exists(bad_file):
        print(f"ERROR: {bad_file} が見つかりません", file=sys.stderr)
        sys.exit(1)

    # パース
    entries = []
    with open(bad_file) as f:
        for line in f:
            parsed = parse_bad_line(line)
            if parsed:
                entries.append(parsed)

    total = len(entries)
    print(f"対象: {eval_dir}")
    print(f"BAD 総数: {total}")

    # ============================================================
    # 1. カテゴリ別内訳
    # ============================================================
    print("\n" + "=" * 60)
    print("1. カテゴリ別内訳")
    print("=" * 60)

    categories = categorize(entries)
    for cat in sorted(categories.keys(), key=lambda c: -len(categories[c])):
        n = len(categories[cat])
        pct = n / total * 100
        print(f"\n## {cat}: {n}件 ({pct:.1f}%)")
        for reading, corpus, akaza in categories[cat][:5]:
            print(f"  {reading}")
            print(f"    期待: {corpus}")
            print(f"    実際: {akaza}")

    # ============================================================
    # 2. 同音異義語の頻出差分パターン
    # ============================================================
    homo_entries = categories.get("同音異義語・その他", [])
    if homo_entries:
        print("\n" + "=" * 60)
        print("2. 同音異義語の頻出差分パターン (上位30)")
        print("=" * 60)
        diff_patterns = analyze_diff_patterns(homo_entries)
        for (c, a), cnt in diff_patterns.most_common(30):
            print(f"  {c} → {a}  ({cnt}回)")

    # ============================================================
    # 3. 1文字差の同音異義語ペア
    # ============================================================
    if homo_entries:
        print("\n" + "=" * 60)
        print("3. 高頻度の1文字差 同音異義語ペア (上位20)")
        print("=" * 60)
        pair_examples = analyze_single_char_pairs(homo_entries)
        for (c, a), examples in sorted(pair_examples.items(), key=lambda x: -len(x[1]))[:20]:
            n = len(examples)
            print(f"\n  {c} → {a}  ({n}回)")
            for reading, corpus, akaza in examples[:2]:
                print(f"    {reading}: {corpus} → {akaza}")

    # ============================================================
    # 4. サブカテゴリ分析
    # ============================================================
    if homo_entries:
        print("\n" + "=" * 60)
        print("4. 同音異義語サブカテゴリ分析")
        print("=" * 60)
        sub_cats = analyze_subcategories(homo_entries)
        for cat in sorted(sub_cats.keys(), key=lambda c: -len(sub_cats[c])):
            n = len(sub_cats[cat])
            print(f"\n### {cat}: {n}件")
            for reading, corpus, akaza in sub_cats[cat][:5]:
                print(f"  {reading}")
                print(f"    期待: {corpus}")
                print(f"    実際: {akaza}")

    # ============================================================
    # 5. accept.tsv 追加候補
    # ============================================================
    print("\n" + "=" * 60)
    print("5. accept.tsv に追加可能な表記揺れの候補")
    print("=" * 60)
    candidates = count_accept_candidates(entries)
    candidate_total = sum(candidates.values())
    print(f"\n合計: {candidate_total}件")
    for pattern, cnt in candidates.most_common():
        label = " + ".join(pattern)
        print(f"  {label}: {cnt}件")

    # ============================================================
    # サマリー
    # ============================================================
    style_count = len(categories.get("表記揺れ（ひらがな⇔漢字・カタカナ）", []))
    punct_count = len(categories.get("全角半角・句読点差", []))
    filterable = style_count + punct_count
    print("\n" + "=" * 60)
    print("サマリー")
    print("=" * 60)
    print(f"  BAD 総数:                  {total}")
    print(f"  accept.tsv でフィルタ可能:  {filterable} ({filterable / total * 100:.1f}%)")
    print(f"  Real BAD (推定):           {total - filterable}")
    print(f"  うち同音異義語:            {len(homo_entries)}")
    print(f"  うち数字関連:              "
          f"{len(categories.get('数字含み文の誤変換', [])) + len(categories.get('数詞→アラビア数字化', []))}")


if __name__ == "__main__":
    main()
