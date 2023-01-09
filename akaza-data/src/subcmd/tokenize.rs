use std::path::Path;

use anyhow::bail;
use lindera::DictionaryKind;
use log::info;

use crate::tokenizer::base::AkazaTokenizer;
use crate::tokenizer::lindera::LinderaTokenizer;
use crate::wikipedia::wikipedia_extracted::ExtractedWikipediaProcessor;

pub fn tokenize(tokenizer_type: &str, src_dir: &str, dst_dir: &str) -> anyhow::Result<()> {
    info!("tokenize({}): {} => {}", tokenizer_type, src_dir, dst_dir);
    let processor = ExtractedWikipediaProcessor::new()?;

    match tokenizer_type {
        "lindera-ipadic" => {
            let tokenizer = LinderaTokenizer::new(DictionaryKind::IPADIC)?;
            processor.process_files(Path::new(src_dir), Path::new(dst_dir), |line| {
                tokenizer.tokenize(line)
            })?;
        }
        _ => bail!("Unknown tokenizer type: {}", tokenizer_type),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use lindera::DictionaryKind::IPADIC;
    use std::fs;

    use crate::tokenizer::base::AkazaTokenizer;

    use super::*;

    #[test]
    #[ignore]
    fn test_wikipedia() -> anyhow::Result<()> {
        let runner = LinderaTokenizer::new(IPADIC)?;
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
