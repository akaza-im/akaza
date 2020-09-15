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

HEPBURNTAB_H = """ぁ      xa      あ      a      ぃ      xi      い      i      ぅ      xu
う      u      う゛      vu      う゛ぁ      va      う゛ぃ      vi      う゛ぇ      ve
う゛ぉ      vo      ぇ      xe      え      e      ぉ      xo      お      o


か      ka      が      ga      き      ki      きゃ      kya      きゅ      kyu
きょ      kyo      ぎ      gi      ぎゃ      gya      ぎゅ      gyu      ぎょ      gyo
く      ku      ぐ      gu      け      ke      げ      ge      こ      ko
ご      go      

さ      sa      ざ      za      し      shi      しゃ      sha      しゅ      shu
し      si      しゃ      sya      しゅ      syu 
しょ      sho      じ      ji      じゃ      ja      じゅ      ju      じょ      jo
しょ      syo      じ      zi      じゃ      zya      じゅ      zyu      じょ      zyo 
す      su      ず      zu      せ      se      ぜ      ze      そ      so
ぞ      zo

た      ta      だ      da      ち      chi      ちゃ      cha      ちゅ      chu
ち      ti      ちゃ      tya      ちゅ      tyu 
ちょ      cho      ぢ      di      ぢゃ      dya      ぢゅ      dyu      ぢょ      dyo
ちょ      tyo

っ      xtsu      っ      xtu 
っう゛      vvu      っう゛ぁ      vva      っう゛ぃ      vvi      
っう゛ぇ      vve      っう゛ぉ      vvo      
っか      kka      っが      gga      っき      kki      っきゃ      kkya      
っきゅ      kkyu      っきょ      kkyo      っぎ      ggi      っぎゃ      ggya      
っぎゅ      ggyu      っぎょ      ggyo      っく      kku      っぐ      ggu      
っけ      kke      っげ      gge      っこ      kko      っご      ggo      っさ      ssa
っざ      zza      っし      sshi      っしゃ      ssha      
っし      ssi      っしゃ      ssya 
っしゅ      sshu      っしょ      ssho      
っしゅ      ssyu      っしょ      ssyo 
っじ      jji      っじゃ      jja      っじゅ      jju      っじょ      jjo      
っじ      zzi      っじゃ      zzya      っじゅ      zzyu      っじょ      zzyo 
っす      ssu      っず      zzu      っせ      sse      っぜ      zze      っそ      sso
っぞ      zzo      った      tta      っだ      dda      っち      cchi      
っち      tti 
っちゃ      ccha      っちゅ      cchu      っちょ      ccho      っぢ      ddi      
っちゃ      ttya      っちゅ      ttyu      っちょ      ttyo
っぢゃ      ddya      っぢゅ      ddyu      っぢょ      ddyo      っつ      ttsu      
っつ      ttu 
っづ      ddu      って      tte      っで      dde      っと      tto      っど      ddo
っは      hha      っば      bba      っぱ      ppa      っひ      hhi      
っひゃ      hhya      っひゅ      hhyu      っひょ      hhyo      っび      bbi      
っびゃ      bbya      っびゅ      bbyu      っびょ      bbyo      っぴ      ppi      
っぴゃ      ppya      っぴゅ      ppyu      っぴょ      ppyo      っふ      ffu      
っふ      hhu 
っふぁ      ffa      っふぃ      ffi      っふぇ      ffe      っふぉ      ffo      
っぶ      bbu      っぷ      ppu      っへ      hhe      っべ      bbe      っぺ      ppe
っほ      hho      っぼ      bbo      っぽ      ppo      っや      yya      っゆ      yyu
っよ      yyo      っら      rra      っり      rri      っりゃ      rrya      
っりゅ      rryu      っりょ      rryo      っる      rru      っれ      rre      
っろ      rro      

つ      tsu      づ      du      て      te      で      de      と      to
つ      tu
ど      do

な      na      に      ni      にゃ      nya      にゅ      nyu      にょ      nyo
ぬ      nu      ね      ne      の      no      

は      ha      ば      ba      ぱ      pa      ひ      hi      ひゃ      hya
ひゅ      hyu      ひょ      hyo      び      bi      びゃ      bya      びゅ      byu
びょ      byo      ぴ      pi      ぴゃ      pya      ぴゅ      pyu      ぴょ      pyo
ふ      fu      ふぁ      fa      ふぃ      fi      ふぇ      fe      ふぉ      fo
ふ      hu
ぶ      bu      ぷ      pu      へ      he      べ      be      ぺ      pe
ほ      ho      ぼ      bo      ぽ      po      

ま      ma      み      mi      みゃ      mya      みゅ      myu      みょ      myo
む      mu      め      me      も      mo

ゃ      xya      や      ya      ゅ      xyu      ゆ      yu      ょ      xyo
よ      yo      

ら      ra      り      ri      りゃ      rya      りゅ      ryu      りょ      ryo
る      ru      れ      re      ろ      ro      

ゎ      xwa      わ      wa      ゐ      wi      ゑ      we
を      wo      ん      n      

ん     n'
でぃ   dyi
ー     -
ちぇ    che ちぇ    tye
っちぇ      cche っちぇ      ttye
じぇ      je じぇ      zye

でゃ dhi でぃ dhi でゅ dhu でぇ dhe でょ dho

てゃ	tha	てぃ	thi	てゅ	thu	てぇ	the	てょ	tho
〜	z-	…	z.  ‥   z,
←	zh	↓	zj	↑	zk	→	zl

。	.	、	,
「	[	」	]
「	z[	」	z]
"""


def pairs(arr, size=2):
    for i in range(0, len(arr) - 1, size):
        yield arr[i:i + size]


ROMKAN_H = {}

for kana, roma in pairs(re.split(r"\s+", HEPBURNTAB_H)):
    ROMKAN_H[roma] = kana

# special modification
# wo -> ヲ, but ヲ/ウォ -> wo
# du -> ヅ, but ヅ/ドゥ -> du
# we -> ウェ, ウェ -> we
ROMKAN_H.update({"du": "づ", "di": "ぢ", "fu": "ふ", "ti": "ち",
                 "wi": "うぃ", "we": "うぇ", "wo": "を"})


# Sort in long order so that a longer Romaji sequence precedes.

def _len_cmp(x):
    return -len(x)


ROMPAT_H = re.compile(
    '(' + "|".join(sorted([re.escape(x) for x in ROMKAN_H.keys()], key=_len_cmp)) + ')'
)


def normalize_double_n(s):
    """
    Normalize double n.
    """

    # Replace double n with n'
    s = re.sub("nn", "n'", s)
    # Remove unnecessary apostrophes
    s = re.sub("n'(?=[^aiueoyn]|$)", "n", s)

    return s


def to_hiragana(s: str) -> str:
    """
    Convert a Romaji (ローマ字) to a Hiragana (平仮名).
    """

    s = s.lower()
    s = normalize_double_n(s)

    # print(ROMKAN_H)
    tmp = ROMPAT_H.sub(lambda x: ROMKAN_H[x.group(1)], s)
    return tmp
