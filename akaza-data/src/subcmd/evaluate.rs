use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::SystemTime;

use anyhow::Context;
use log::info;

use libakaza::config::{DictConfig, DictEncoding, DictType, DictUsage, EngineConfig};
use libakaza::engine::base::HenkanEngine;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;
use libakaza::graph::reranking::ReRankingWeights;

#[derive(Default)]
struct SaigenRitsu {
    /// total_lcs = N_{LCS}
    /// LCS(最長共通部分列)の文字数の和。
    /// https://www.anlp.jp/proceedings/annual_meeting/2011/pdf_dir/C4-6.pdf
    total_lcs: usize,
    /// 一括変換結果の文字数の和。
    /// N_{sys}
    total_sys: usize,
}

impl SaigenRitsu {
    /// @param teacher コーパスにあるの変換結果
    /// @param my_candidate 評価対象モデルにより出力された変換結果
    fn add(&mut self, teacher: &str, my_candidate: &str) {
        let teacher: Vec<char> = teacher.chars().collect();
        let my_candidate: Vec<char> = my_candidate.chars().collect();
        let lcs = lcs::LcsTable::new(&teacher, &my_candidate);
        let lcs = lcs.longest_common_subsequence();
        self.total_lcs += lcs.len();
        self.total_sys += my_candidate.len();
    }

    fn merge(&mut self, other: &SaigenRitsu) {
        self.total_lcs += other.total_lcs;
        self.total_sys += other.total_sys;
    }

    fn rate(&self) -> f32 {
        100.0 * (self.total_lcs as f32) / (self.total_sys as f32)
    }
}

/// 全角数字を半角に正規化する
fn normalize_fullwidth_numbers(s: &str) -> String {
    s.replace('０', "0")
        .replace('１', "1")
        .replace('２', "2")
        .replace('３', "3")
        .replace('４', "4")
        .replace('５', "5")
        .replace('６', "6")
        .replace('７', "7")
        .replace('８', "8")
        .replace('９', "9")
}

/// コーパスファイルをパースして (yomi, surface) のペアを返す
fn parse_corpus_file(path: &str) -> anyhow::Result<Vec<(String, String)>> {
    let fp = File::open(path).with_context(|| format!("File: {path}"))?;
    let mut lines = Vec::new();
    for line in BufReader::new(fp).lines() {
        let line = line?;
        let line = line.trim().to_string();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let (yomi, surface) = line
            .split_once(' ')
            .with_context(|| format!("source: {line}"))
            .unwrap();
        let yomi = normalize_fullwidth_numbers(&yomi.replace('|', ""));
        let surface = normalize_fullwidth_numbers(&surface.replace('|', ""));
        lines.push((yomi, surface));
    }
    Ok(lines)
}

struct MismatchEntry {
    yomi: String,
    surface: String,
    got: String,
    in_topk: bool,
}

struct EvalResult {
    good_cnt: usize,
    topk_cnt: usize,
    bad_cnt: usize,
    saigen_ritsu: SaigenRitsu,
    mismatches: Vec<MismatchEntry>,
}

/// モデル/変換アルゴリズムを評価する。
///
/// 日本語かな漢字変換における識別モデルの適用とその考察
/// https://www.anlp.jp/proceedings/annual_meeting/2011/pdf_dir/C4-6.pdf
///
/// にのっている評価方法を採用。
///
/// なぜこうしているかというと、mozc の論文にのっている BLEU を使用する方式より実装が楽だからです!
pub fn evaluate(
    corpus: &Vec<String>,
    eucjp_dict: &Vec<String>,
    utf8_dict: &Vec<String>,
    model_dir: String,
    k_best: usize,
    reranking_weights: ReRankingWeights,
) -> anyhow::Result<()> {
    let mut dicts: Vec<DictConfig> = Vec::new();
    for path in eucjp_dict {
        dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::EucJp,
            path: path.clone(),
            usage: DictUsage::Normal,
        })
    }

    for path in utf8_dict {
        dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::Utf8,
            path: path.clone(),
            usage: DictUsage::Normal,
        })
    }

    let config = EngineConfig {
        dicts,
        model: model_dir,
        dict_cache: false,
        reranking_weights,
    };

    // コーパスの全行を事前に読み込む
    let mut lines: Vec<(String, String)> = Vec::new();
    for file in corpus {
        lines.extend(parse_corpus_file(file)?);
    }

    let total_t1 = SystemTime::now();

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let chunk_size = lines.len().div_ceil(num_threads);

    let results: Vec<anyhow::Result<EvalResult>> = std::thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(chunk_size)
            .map(|chunk| {
                let config = config.clone();
                s.spawn(move || {
                    let engine = BigramWordViterbiEngineBuilder::new(config).build()?;
                    let force_ranges = Vec::new();

                    let mut good_cnt = 0;
                    let mut topk_cnt = 0;
                    let mut bad_cnt = 0;
                    let mut saigen_ritsu = SaigenRitsu::default();
                    let mut mismatches = Vec::new();

                    for (yomi, surface) in chunk {
                        // convert_k_best でリランキング適用済みの結果を取得し、
                        // 先頭パスを 1-best として使用する
                        let k_results =
                            engine.convert_k_best(yomi.as_str(), Some(&force_ranges), k_best)?;
                        let got = k_results
                            .first()
                            .map(|p| {
                                p.segments
                                    .iter()
                                    .map(|s| s[0].surface.clone())
                                    .collect::<String>()
                            })
                            .unwrap_or_default();

                        saigen_ritsu.add(surface, &got);

                        if *surface == got {
                            info!("{} => (teacher={}, akaza={})", yomi, surface, got);
                            good_cnt += 1;
                        } else {
                            let in_topk = k_results.iter().any(|path| {
                                let s: String = path
                                    .segments
                                    .iter()
                                    .map(|seg| seg[0].surface.clone())
                                    .collect();
                                s == *surface
                            });

                            if in_topk {
                                topk_cnt += 1;
                            } else {
                                bad_cnt += 1;
                            }

                            mismatches.push(MismatchEntry {
                                yomi: yomi.clone(),
                                surface: surface.clone(),
                                got,
                                in_topk,
                            });
                        }
                    }

                    Ok(EvalResult {
                        good_cnt,
                        topk_cnt,
                        bad_cnt,
                        saigen_ritsu,
                        mismatches,
                    })
                })
            })
            .collect();

        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    // 結果を集約
    let mut good_cnt = 0;
    let mut topk_cnt = 0;
    let mut bad_cnt = 0;
    let mut saigen_ritsu = SaigenRitsu::default();

    for result in results {
        let result = result?;
        good_cnt += result.good_cnt;
        topk_cnt += result.topk_cnt;
        bad_cnt += result.bad_cnt;
        saigen_ritsu.merge(&result.saigen_ritsu);

        for m in &result.mismatches {
            if m.in_topk {
                println!(
                    "[TOP-{}] {} => corpus={}, akaza={}",
                    k_best, m.yomi, m.surface, m.got
                );
            } else {
                println!("[BAD] {} => corpus={}, akaza={}", m.yomi, m.surface, m.got);
            }
        }
    }

    let total_t2 = SystemTime::now();
    let total_elapsed = total_t2.duration_since(total_t1)?;

    info!(
        "Good={}, Top-{}={}, Bad={}, elapsed={}ms, 再現率={}",
        good_cnt,
        k_best,
        topk_cnt,
        bad_cnt,
        total_elapsed.as_millis(),
        saigen_ritsu.rate(),
    );

    Ok(())
}
