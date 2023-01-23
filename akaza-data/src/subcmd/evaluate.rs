use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::SystemTime;

use anyhow::Context;
use log::info;

use libakaza::config::{Config, DictConfig};
use libakaza::engine::base::HenkanEngine;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;

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

    fn rate(&self) -> f32 {
        100.0 * (self.total_lcs as f32) / (self.total_sys as f32)
    }
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
    model_dir: Option<String>,
    load_user_config: bool,
) -> anyhow::Result<()> {
    let mut dicts: Vec<DictConfig> = Vec::new();
    for path in eucjp_dict {
        dicts.push(DictConfig {
            dict_type: "skk".to_string(),
            encoding: Some("euc-jp".to_string()),
            path: path.clone(),
        })
    }

    for path in utf8_dict {
        dicts.push(DictConfig {
            dict_type: "skk".to_string(),
            encoding: Some("utf-8".to_string()),
            path: path.clone(),
        })
    }

    let mut builder = BigramWordViterbiEngineBuilder::new(Config {
        dicts,
        single_term: Default::default(),
        romkan: None,
        keymap: None,
        model: None,
    });
    builder.load_user_config(load_user_config);
    if let Some(model_dir) = model_dir {
        builder.model_dir(&model_dir);
    }
    let akaza = builder.build()?;

    let mut good_cnt = 0;
    let mut bad_cnt = 0;

    let force_ranges = Vec::new();
    let total_t1 = SystemTime::now();

    let mut saigen_ritsu = SaigenRitsu::default();

    for file in corpus {
        let fp = File::open(file).with_context(|| format!("File: {}", file))?;
        for line in BufReader::new(fp).lines() {
            let line = line?;
            let line = line.trim();
            if line.starts_with('#') {
                continue; // comment行
            }

            let (yomi, surface) = line
                .split_once(' ')
                .with_context(|| format!("source: {}", line))
                .unwrap();
            let yomi = yomi.replace('|', "");
            let surface = surface.replace('|', "");

            let t1 = SystemTime::now();
            let result = akaza.convert(yomi.as_str(), Some(&force_ranges))?;
            let t2 = SystemTime::now();
            let elapsed = t2.duration_since(t1)?;

            let terms: Vec<String> = result.iter().map(|f| f[0].surface.clone()).collect();
            let got = terms.join("");

            // 最長共通部分列を算出。
            saigen_ritsu.add(&surface, &got);

            if surface == got {
                info!("{} => (teacher={}, akaza={})", yomi, surface, got);
                good_cnt += 1;
            } else {
                println!(
                    "{} =>\n\
                   |  corpus={}\n\
                   |  akaza ={}\n\
                   Good count={} bad count={} elapsed={}ms saigen={}",
                    yomi,
                    surface,
                    got,
                    good_cnt,
                    bad_cnt,
                    elapsed.as_millis(),
                    saigen_ritsu.rate()
                );

                // 遅いなと思ったら cargo run --release になってるか確認すべし
                // https://codom.hatenablog.com/entry/2017/06/03/221318

                bad_cnt += 1;
            }
        }
    }

    let total_t2 = SystemTime::now();
    let total_elapsed = total_t2.duration_since(total_t1)?;

    info!(
        "Good count={} bad count={}, elapsed={}ms, 再現率={}",
        good_cnt,
        bad_cnt,
        total_elapsed.as_millis(),
        saigen_ritsu.rate(),
    );

    Ok(())
}
