# -*- coding: utf-8 -*-

# This script is based on Ruby/Romkan and python-romkan.

#     Ruby/Romkan - a Romaji <-> Kana conversion library for Ruby.
#    
#     Copyright (C) 2001 Satoru Takabayashi <satoru@namazu.org>
#         All rights reserved.
#         This is free software with ABSOLUTELY NO WARRANTY.
#    
#     You can redistribute it and/or modify it under the terms of 
#     the Ruby's licence.
#
#   The BSD License

#   Copyright (c) 2012, 2013 Mort Yao <mort.yao@gmail.com>
#   Copyright (c) 2010 Masato Hagiwara <hagisan@gmail.com>
#   Copyright (c) 2001 Satoru Takabayashi <satoru@namazu.org>
#   All rights reserved.

#   Redistribution and use in source and binary forms, with or without
#   modification, are permitted provided that the following conditions are met:
#       * Redistributions of source code must retain the above copyright
#       notice, this list of conditions and the following disclaimer.
#       * Redistributions in binary form must reproduce the above copyright
#       notice, this list of conditions and the following disclaimer in the
#       documentation and/or other materials provided with the distribution.
#       * Neither the name of the <organization> nor the
#       names of its contributors may be used to endorse or promote products
#       derived from this software without specific prior written permission.

#   THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
#   ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
#   WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
#   DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY
#   DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
#   (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
#   LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
#   ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
#   (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
#   SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

from __future__ import unicode_literals

import re

# This table is imported from KAKASI <http://kakasi.namazu.org/> and modified.

ROMKAN_H = {
    "xa": "ぁ", "a": "あ", "xi": "ぃ", "i": "い", "xu": "ぅ", "u": "う", "vu": "う゛",
    "va": "う゛ぁ", "vi": "う゛ぃ", "ve": "う゛ぇ", "vo": "う゛ぉ",
    "xe": "ぇ", "e": "え",
    "xo": "ぉ", "o": "お",

    "ka": "か",
    "ga": "が",
    "ki": "き",
    "kya": "きゃ", "kyu": "きゅ", "kyo": "きょ",
    "gi": "ぎ", "gya": "ぎゃ", "gyu": "ぎゅ", "gyo": "ぎょ",
    "ku": "く", "gu": "ぐ", "ke": "け", "ge": "げ", "ko": "こ", "go": "ご",
    "sa": "さ", "za": "ざ",
    "shi": "し", "sha": "しゃ", "shu": "しゅ", "si": "し", "sya": "しゃ", "syu": "しゅ", "sho": "しょ",
    "ji": "じ", "ja": "じゃ", "ju": "じゅ", "jo": "じょ", "syo": "しょ", "zi": "じ", "zya": "じゃ", "zyu": "じゅ",
    "zyo": "じょ",
    "su": "す", "zu": "ず",
    "se": "せ", "ze": "ぜ",
    "so": "そ", "zo": "ぞ",
    "ta": "た", "da": "だ",
    "chi": "ち", "cha": "ちゃ", "chu": "ちゅ", "ti": "ち", "tya": "ちゃ", "tyu": "ちゅ", "cho": "ちょ",
    "di": "ぢ", "dya": "ぢゃ", "dyu": "ぢゅ", "dyo": "ぢょ", "tyo": "ちょ",
    "xtsu": "っ", "xtu": "っ",
    "vvu": "っう゛", "vva": "っう゛ぁ", "vvi": "っう゛ぃ", "vve": "っう゛ぇ", "vvo": "っう゛ぉ",
    "kka": "っか", "gga": "っが",
    "kki": "っき", "kkya": "っきゃ", "kkyu": "っきゅ", "kkyo": "っきょ",
    "ggi": "っぎ", "ggya": "っぎゃ", "ggyu": "っぎゅ", "ggyo": "っぎょ",
    "kku": "っく", "ggu": "っぐ",
    "kke": "っけ", "gge": "っげ",
    "kko": "っこ", "ggo": "っご",
    "ssa": "っさ", "zza": "っざ",
    "sshi": "っし", "ssha": "っしゃ",
    "ssi": "っし", "ssya": "っしゃ", "sshu": "っしゅ", "ssho": "っしょ", "ssyu": "っしゅ", "ssyo": "っしょ",
    "jji": "っじ", "jja": "っじゃ", "jju": "っじゅ", "jjo": "っじょ",
    "zzi": "っじ", "zzya": "っじゃ", "zzyu": "っじゅ", "zzyo": "っじょ",
    "ssu": "っす", "zzu": "っず",
    "sse": "っせ", "zze": "っぜ",
    "sso": "っそ", "zzo": "っぞ",
    "tta": "った", "dda": "っだ",
    "cchi": "っち", "tti": "っち",
    "ccha": "っちゃ", "cchu": "っちゅ", "ccho": "っちょ",
    "ddi": "っぢ", "ttya": "っちゃ", "ttyu": "っちゅ", "ttyo": "っちょ",
    "ddya": "っぢゃ", "ddyu": "っぢゅ", "ddyo": "っぢょ",
    "ttsu": "っつ", "ttu": "っつ", "ddu": "っづ",
    "tte": "って", "dde": "っで",
    "tto": "っと",
    "ddo": "っど",
    "hha": "っは", "bba": "っば", "ppa": "っぱ",
    "hhi": "っひ", "hhya": "っひゃ", "hhyu": "っひゅ", "hhyo": "っひょ",
    "bbi": "っび", "bbya": "っびゃ", "bbyu": "っびゅ", "bbyo": "っびょ",
    "ppi": "っぴ", "ppya": "っぴゃ", "ppyu": "っぴゅ", "ppyo": "っぴょ",
    "ffu": "っふ", "hhu": "っふ", "ffa": "っふぁ", "ffi": "っふぃ", "ffe": "っふぇ", "ffo": "っふぉ",
    "bbu": "っぶ", "ppu": "っぷ",
    "hhe": "っへ", "bbe": "っべ", "ppe": "っぺ",
    "hho": "っほ", "bbo": "っぼ", "ppo": "っぽ",
    "yya": "っや", "yyu": "っゆ", "yyo": "っよ",
    "rra": "っら",
    "rri": "っり", "rrya": "っりゃ", "rryu": "っりゅ", "rryo": "っりょ",
    "rru": "っる",
    "rre": "っれ",
    "rro": "っろ",
    "tu": "つ", "tsu": "つ", "du": "づ",
    "te": "て", "de": "で",
    "to": "と",
    "do": "ど",
    "na": "な",
    "ni": "に", "nya": "にゃ", "nyu": "にゅ", "nyo": "にょ",
    "nu": "ぬ",
    "ne": "ね",
    "no": "の",
    "ha": "は", "ba": "ば", "pa": "ぱ",
    "hi": "ひ", "hya": "ひゃ", "hyu": "ひゅ", "hyo": "ひょ",
    "bi": "び", "bya": "びゃ", "byu": "びゅ", "byo": "びょ",
    "pi": "ぴ", "pya": "ぴゃ", "pyu": "ぴゅ", "pyo": "ぴょ",
    "fu": "ふ", "fa": "ふぁ", "fi": "ふぃ", "fe": "ふぇ", "fo": "ふぉ",
    "hu": "ふ", "bu": "ぶ", "pu": "ぷ",
    "he": "へ", "be": "べ", "pe": "ぺ",
    "ho": "ほ", "bo": "ぼ", "po": "ぽ",
    "ma": "ま",
    "mi": "み", "mya": "みゃ", "myu": "みゅ", "myo": "みょ",
    "mu": "む",
    "me": "め",
    "mo": "も",
    "xya": "ゃ", "ya": "や",
    "xyu": "ゅ", "yu": "ゆ",
    "xyo": "ょ", "yo": "よ",
    "ra": "ら",
    "ri": "り", "rya": "りゃ", "ryu": "りゅ", "ryo": "りょ",
    "ru": "る",
    "re": "れ",
    "ro": "ろ",
    "xwa": "ゎ", "wa": "わ",
    "wo": "を",
    "n": "ん", "n'": "ん",
    "dyi": "でぃ",
    "-": "ー",
    "che": "ちぇ", "tye": "ちぇ",
    "cche": "っちぇ", "ttye": "っちぇ",
    "je": "じぇ", "zye": "じぇ",
    "dha": "でゃ", "dhi": "でぃ", "dhu": "でゅ", "dhe": "でぇ", "dho": "でょ",
    "tha": "てゃ", "thi": "てぃ", "thu": "てゅ", "the": "てぇ", "tho": "てょ",

    ".": "。", ",": "、", "[": "「", "]": "」", "z[": "『",

    "z-": "〜", "z.": "…", "z,": "‥", "zh": "←", "zj": "↓", "zk": "↑", "zl": "→",
    "z]": "』", "z/": "・",

    "wi": "うぃ", "we": "うぇ",
}


# Sort in long order so that a longer Romaji sequence precedes.

def _len_cmp(x):
    return -len(x)


def normalize_double_n(s):
    """
    Normalize double n.
    """

    # Replace double n with n'
    s = re.sub("nn", "n'", s)
    # Remove unnecessary apostrophes
    s = re.sub("n'(?=[^aiueoyn]|$)", "n", s)

    return s


class RomkanConverter:
    def __init__(self):
        self.pattern = re.compile(
            '(' + "|".join(sorted([re.escape(x) for x in ROMKAN_H.keys()], key=_len_cmp)) + ')'
        )

    def to_hiragana(self, s: str) -> str:
        """
        Convert a Romaji (ローマ字) to a Hiragana (平仮名).
        """

        s = s.lower()
        s = normalize_double_n(s)

        tmp = self.pattern.sub(lambda x: ROMKAN_H[x.group(1)], s)
        return tmp


def to_hiragana(s: str) -> str:
    """
    Convert a Romaji (ローマ字) to a Hiragana (平仮名).
    """

    return RomkanConverter().to_hiragana(s)
