use std::io::{self, BufRead};

use log::info;

use crate::tokenizer::base::AkazaTokenizer;
use crate::tokenizer::vibrato::VibratoTokenizer;

/// 一行の自然文を `surface/yomi` 形式で出力する。
/// `text` が `Some` の場合は引数の1行を処理し、`None` の場合は stdin から行ごとに処理する。
pub fn tokenize_line(
    system_dict: &str,
    user_dict: Option<String>,
    kana_preferred: bool,
    text: Option<String>,
) -> anyhow::Result<()> {
    let tokenizer = VibratoTokenizer::new(system_dict, user_dict)?;

    match text {
        Some(text) => {
            info!("tokenize-line: {}", text);
            let annotated = tokenizer.tokenize(&text, kana_preferred)?;
            println!("{annotated}");
        }
        None => {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line?;
                info!("tokenize-line: {}", line);
                let annotated = tokenizer.tokenize(&line, kana_preferred)?;
                println!("{annotated}");
            }
        }
    }

    Ok(())
}
