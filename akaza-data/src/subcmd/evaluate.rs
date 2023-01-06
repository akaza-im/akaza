use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::SystemTime;

use anyhow::Context;
use log::info;

use libakaza::akaza_builder::AkazaBuilder;

pub fn evaluate(corpus_dir: &String, system_data_dir: &str) -> anyhow::Result<()> {
    /*
    # corpus.0.txt デバッグ用のファイル
    # corpus.1.txt メイン(候補割り当ても含む)
    # corpus.2.txt テストセットに対する入力
    # corpus.3.txt メイン(もらいもの)
    # corpus.4.txt 誤変換
    # corpus.5.txt 出どころ不明
        */
    let files = [
        "corpus.0.txt",
        "corpus.1.txt",
        "corpus.2.txt",
        "corpus.3.txt",
        "corpus.4.txt",
        "corpus.5.txt",
    ];

    let akaza = AkazaBuilder::default()
        .system_data_dir(system_data_dir)
        .build()?;

    let mut good_cnt = 0;
    let mut bad_cnt = 0;

    let force_ranges = Vec::new();
    let total_t1 = SystemTime::now();

    for file in files {
        let fp = File::open(corpus_dir.to_string() + "/" + file)
            .with_context(|| format!("File: {}/{}", corpus_dir, file))?;
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
            let result = akaza.convert(yomi.as_str(), &force_ranges)?;
            let t2 = SystemTime::now();
            let elapsed = t2.duration_since(t1)?;

            let terms: Vec<String> = result.iter().map(|f| f[0].kanji.clone()).collect();
            let got = terms.join("");

            if surface == got {
                info!("{} => (teacher={}, akaza={})", yomi, surface, got);
                good_cnt += 1;
            } else {
                println!(
                    "{} =>\n\
                   |  corpus={}\n\
                   |  akaza ={}\n\
                   Good count={} bad count={} elapsed={}ms",
                    yomi,
                    surface,
                    got,
                    good_cnt,
                    bad_cnt,
                    elapsed.as_millis()
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
        "Good count={} bad count={}, elapsed={}ms",
        good_cnt,
        bad_cnt,
        total_elapsed.as_millis()
    );

    Ok(())
}
