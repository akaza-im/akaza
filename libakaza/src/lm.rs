pub mod system_bigram;
pub mod system_unigram_lm;

// ↓↓このあたりは C++ 時代の Spec。
// rust 版ではいったん、簡単のために id を sizeof(usize) バイト書いている。
// TODO 最適化する

/*

-- 1gram

    {word} # in utf-8
    0xff   # marker
    packed ID     # 3 bytes(24bit). 最大語彙: 8,388,608(2**24/2)
    packed float  # score: 4 bytes

-- 2gram

    {word1 ID}    # 3 bytes
    {word2 ID}    # 3 bytes
    packed float  # score: 4 bytes

 */
