use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use anyhow::Context;

use libakaza::config::{Config, DictConfig, DictEncoding, DictType, DictUsage};
use libakaza::engine::base::HenkanEngine;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;

pub struct BenchOptions<'a> {
    pub corpus: &'a [String],
    pub model_dir: Option<&'a str>,
    pub eucjp_dict: &'a [String],
    pub utf8_dict: &'a [String],
    pub max_sentences: usize,
    pub k: usize,
}

/// インクリメンタル変換のベンチマークを実行する。
///
/// コーパスから読みを取得し、1文字ずつひらがなを増やしながら
/// convert_k_best() を呼び出してレイテンシを計測する。
pub fn bench(opts: BenchOptions) -> anyhow::Result<()> {
    // --- 設定読み込み ---
    let mut config = Config::load()?;

    if let Some(dir) = opts.model_dir {
        config.engine.model = dir.to_string();
    }

    for path in opts.eucjp_dict {
        config.engine.dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::EucJp,
            path: path.clone(),
            usage: DictUsage::Normal,
        });
    }
    for path in opts.utf8_dict {
        config.engine.dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::Utf8,
            path: path.clone(),
            usage: DictUsage::Normal,
        });
    }

    config.engine.dict_cache = false;

    // --- エンジン構築 ---
    let engine_t1 = Instant::now();
    let engine = BigramWordViterbiEngineBuilder::new(config.engine).build()?;
    let engine_elapsed = engine_t1.elapsed();
    println!("Engine built in {}ms", engine_elapsed.as_millis());

    // --- コーパス読み込み ---
    let mut sentences: Vec<String> = Vec::new();
    for file in opts.corpus {
        let fp = File::open(file).with_context(|| format!("File: {file}"))?;
        for line in BufReader::new(fp).lines() {
            let line = line?;
            let line = line.trim().to_string();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let (yomi, _surface) = line
                .split_once(' ')
                .with_context(|| format!("source: {line}"))?;
            let yomi = yomi.replace('|', "");
            if yomi.is_empty() {
                continue;
            }
            sentences.push(yomi);
            if sentences.len() >= opts.max_sentences {
                break;
            }
        }
        if sentences.len() >= opts.max_sentences {
            break;
        }
    }

    println!("Loaded {} sentences from corpus", sentences.len());
    println!("---");

    // --- ベンチマーク実行 ---
    let mut all_durations_us: Vec<u64> = Vec::new();

    for (i, yomi) in sentences.iter().enumerate() {
        let chars: Vec<char> = yomi.chars().collect();
        let num_chars = chars.len();
        let mut sentence_durations_us: Vec<u64> = Vec::new();

        for end in 1..=num_chars {
            let partial: String = chars[..end].iter().collect();
            let t1 = Instant::now();
            let _ = engine.convert_k_best(&partial, None, opts.k)?;
            let elapsed = t1.elapsed();
            let us = elapsed.as_micros() as u64;
            sentence_durations_us.push(us);
        }

        let total_us: u64 = sentence_durations_us.iter().sum();
        let max_us = sentence_durations_us.iter().copied().max().unwrap_or(0);
        let avg_us = if sentence_durations_us.is_empty() {
            0.0
        } else {
            total_us as f64 / sentence_durations_us.len() as f64
        };

        println!(
            "[{:>3}/{}] {} ({} chars, {} conversions)",
            i + 1,
            sentences.len(),
            yomi,
            num_chars,
            num_chars
        );
        println!(
            "          avg={:.1}ms, max={:.1}ms, total={:.1}ms",
            avg_us / 1000.0,
            max_us as f64 / 1000.0,
            total_us as f64 / 1000.0
        );

        all_durations_us.extend_from_slice(&sentence_durations_us);
    }

    // --- サマリー ---
    println!("===");
    let total_conversions = all_durations_us.len();
    if total_conversions == 0 {
        println!("No conversions performed.");
        return Ok(());
    }

    all_durations_us.sort();

    let total_us: u64 = all_durations_us.iter().sum();
    let avg_us = total_us as f64 / total_conversions as f64;
    let median_us = all_durations_us[total_conversions / 2];
    let p95_us = all_durations_us[(total_conversions as f64 * 0.95) as usize];
    let p99_us = all_durations_us[(total_conversions as f64 * 0.99) as usize];
    let max_us = all_durations_us[total_conversions - 1];

    println!(
        "Summary: {} sentences, {} incremental conversions",
        sentences.len(),
        total_conversions
    );
    println!(
        "  avg={:.1}ms, median={:.1}ms, p95={:.1}ms, p99={:.1}ms, max={:.1}ms",
        avg_us / 1000.0,
        median_us as f64 / 1000.0,
        p95_us as f64 / 1000.0,
        p99_us as f64 / 1000.0,
        max_us as f64 / 1000.0
    );
    println!("  total conversion time: {:.1}ms", total_us as f64 / 1000.0);

    Ok(())
}
