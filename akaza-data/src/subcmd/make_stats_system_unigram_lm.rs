use std::cmp::max;
use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use chrono::Local;
use log::info;

use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLMBuilder;

/// 統計的かな漢字変換のためのユニグラムシステム言語モデルの作成
///
/// wfreq ファイルを開いてパースし、ユニグラム言語モデルファイルを作成して保存する。
pub fn make_stats_system_unigram_lm(srcpath: &str, dstpath: &str) -> anyhow::Result<()> {
    // 16 はヒューリスティックな値。調整の余地。
    let threshold = 16_u32;

    let mut wordcnt = parse_wfreq(srcpath, threshold)?;
    if wordcnt.len() >= 8388608 {
        // edge cost 言語モデルファイルの容量を小さく保つために
        // 3 byte に ID が収めるようにする。
        // よって、最大でも 8,388,608 単語までになるように vocab を制限する。
        // 現実的な線で切っても、500万単語ぐらいで十分。
        panic!("too much words in wfreq file: {}", srcpath);
    }

    homograph_hack(&mut wordcnt);
    score_hack(&mut wordcnt);

    let scoremap = make_score_map(&wordcnt);

    let mut builder = MarisaSystemUnigramLMBuilder::default();
    for (word, score) in &scoremap {
        builder.add(word.as_str(), *score);
    }

    // 総出現単語数
    let c = wordcnt.values().sum();
    // 単語の種類数
    let v = wordcnt.keys().count();
    builder.set_default_cost_for_short(calc_score(1, c, v));
    builder.set_default_cost(calc_score(0, c, v));
    info!("Score for word count 1: {}", calc_score(1, c, v));
    info!("Score for word count 0: {}", calc_score(0, c, v));

    println!("Writing {}", dstpath);
    builder.save(dstpath)?;

    let dumpfname = format!(
        "work/dump/unigram-{}.txt",
        Local::now().format("%Y%m%d-%H%M%S")
    );
    println!("Dump to text file: {}", dumpfname);
    let mut file = File::create(dumpfname)?;
    for (word, score) in scoremap {
        file.write_fmt(format_args!("{}\t{}\n", word, score))?;
    }

    Ok(())
}

fn homograph_hack(wordcnt: &mut HashMap<String, u32>) {
    // 同形異音字の処理
    // mecab では "日本" は "日本/にほん" に処理されるため、日本/にっぽん が表出しない。
    // かな漢字変換上は、同一程度の確率で出るだろうと予想されることから、この2つの確率を同じに設定する。
    {
        let (src, dst) = ("日本/にほん", "日本/にっぽん");
        try_copy_cost(src, dst, wordcnt);
        try_copy_cost(dst, src, wordcnt);
    }
}

fn try_copy_cost(word1: &str, word2: &str, wordcnt: &mut HashMap<String, u32>) {
    if !wordcnt.contains_key(word2) {
        if let Some(cost) = wordcnt.get(word1) {
            wordcnt.insert(word2.to_string(), *cost);
        }
    }
}

// Wikipedia 特有で、日本語の一般的な分布よりも少しずれたスコアをつけている時があるので
// ヒューリスティックに調整する。
fn score_hack(wordcnt: &mut HashMap<String, u32>) {
    // a の方のスコアが b よりも高くなるように調整します。
    // https://github.com/tokuhirom/akaza/wiki/%E5%A4%A7%E5%AD%97
    // https://github.com/tokuhirom/akaza/wiki/%E5%8D%BF
    for (a, b) in [("今日/きょう", "卿/きょう"), ("大事/だいじ", "大字/だいじ")]
    {
        let Some(a_score) = wordcnt.get(a) else {
            return;
        };
        let Some(b_score) = wordcnt.get(b) else {
            return;
        };
        wordcnt.insert(a.to_string(), max(*a_score, b_score + 1));
    }
}

fn make_score_map(wordcnt: &HashMap<String, u32>) -> HashMap<String, f32> {
    // 総出現単語数
    let c = wordcnt.values().sum();
    // 単語の種類数
    let v = wordcnt.keys().count();
    wordcnt
        .iter()
        .map(|(word, cnt)| (word.clone(), calc_score(*cnt, c, v)))
        .collect::<HashMap<_, _>>()
}

pub fn calc_score(n_words: u32, c: u32, v: usize) -> f32 {
    let alpha = 0.00001;
    -f32::log10(((n_words as f32) + alpha) / ((c as f32) + alpha * (v as f32)))
}

fn parse_wfreq(src_file: &str, threshold: u32) -> anyhow::Result<HashMap<String, u32>> {
    let file = File::open(src_file)?;
    let mut map: HashMap<String, u32> = HashMap::new();

    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let (word, cnt) = line.trim().split_once('\t').unwrap();
        let cnt: u32 = cnt.parse().unwrap();
        if cnt > threshold {
            map.insert(word.to_string(), cnt);
        }
    }
    Ok(map)
}
