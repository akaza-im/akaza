use std::path::{Path, PathBuf};

use anyhow::bail;
use lindera::DictionaryKind;
use log::info;
use rayon::prelude::*;

use crate::tokenizer::base::AkazaTokenizer;
use crate::tokenizer::lindera::LinderaTokenizer;
use crate::tokenizer::vibrato::VibratoTokenizer;
use crate::wikipedia::wikipedia_extracted::ExtractedWikipediaProcessor;

pub fn tokenize(
    tokenizer_type: &str,
    user_dict: Option<String>,
    src_dir: &str,
    dst_dir: &str,
) -> anyhow::Result<()> {
    // ここのコピー&ペーストは rust 力が高ければなんとかなりそう。
    info!("tokenize({}): {} => {}", tokenizer_type, src_dir, dst_dir);
    let processor = ExtractedWikipediaProcessor::new()?;

    match tokenizer_type {
        "lindera-ipadic" => {
            let tokenizer =
                LinderaTokenizer::new(DictionaryKind::IPADIC, user_dict.map(|f| PathBuf::from(f)))?;
            let file_list = processor.get_file_list(Path::new(src_dir), Path::new(dst_dir))?;

            let result = file_list
                .par_iter()
                .map(|(src, dst)| {
                    info!("GOT: {:?} {:?}", src, dst);
                    processor.process_file(
                        Path::new(src),
                        Path::new(dst),
                        &mut (|f| tokenizer.tokenize(f)),
                    )
                })
                .collect::<Vec<_>>();
            for r in result {
                r.unwrap();
            }
        }
        "vibrato-ipadic" => {
            let tokenizer = VibratoTokenizer::new()?;
            let file_list = processor.get_file_list(Path::new(src_dir), Path::new(dst_dir))?;

            let result = file_list
                .par_iter()
                .map(|(src, dst)| {
                    info!("GOT: {:?} {:?}", src, dst);
                    processor.process_file(
                        Path::new(src),
                        Path::new(dst),
                        &mut (|f| tokenizer.tokenize(f)),
                    )
                })
                .collect::<Vec<_>>();
            for r in result {
                r.unwrap();
            }
        }
        _ => bail!("Unknown tokenizer type: {}", tokenizer_type),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use lindera::DictionaryKind::IPADIC;

    use crate::tokenizer::base::AkazaTokenizer;

    use super::*;

    #[test]
    #[ignore]
    fn test_wikipedia() -> anyhow::Result<()> {
        let runner = LinderaTokenizer::new(IPADIC, None)?;
        let processor = ExtractedWikipediaProcessor::new()?;

        let fname = "work/extracted/BE/wiki_02";
        fs::create_dir_all("work/mecab/wikipedia-annotated/BE/")?;
        processor.process_file(
            Path::new(fname),
            Path::new("work/mecab/wikipedia-annotated/BE/wiki_02"),
            &mut (|line| runner.tokenize(line)),
        )?;
        Ok(())
    }

    // #[test]
    // #[ignore]
    // fn test_all() -> anyhow::Result<()> {
    //     let _ = env_logger::builder().is_test(true).try_init();
    //     annotate_wikipedia()?;
    //     Ok(())
    // }
}
