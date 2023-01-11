use std::path::{Path, PathBuf};

use lindera::DictionaryKind;
use log::info;
use rayon::prelude::*;

use crate::tokenizer::base::AkazaTokenizer;
use crate::tokenizer::lindera::LinderaTokenizer;
use crate::tokenizer::vibrato::VibratoTokenizer;
use crate::wikipedia::wikipedia_extracted::ExtractedWikipediaProcessor;
use crate::aozora_bunko::aozora_bunko::AozoraBunkoProcessor;

pub fn tokenize_aozora_bunko_vibrato_ipadic(
    system_dict: String,
    user_dict: Option<String>,
    src_dir: &str,
    dst_dir: &str,
) -> anyhow::Result<()> {
    info!("tokenize: {} => {}", src_dir, dst_dir);
    let processor = AozoraBunkoProcessor::new()?;

    let tokenizer = VibratoTokenizer::new(system_dict.as_str(), user_dict)?;
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

    processor.write_success_file(Path::new(dst_dir))?;

    Ok(())
}

pub fn tokenize_vibrato_ipadic(
    system_dict: String,
    user_dict: Option<String>,
    src_dir: &str,
    dst_dir: &str,
) -> anyhow::Result<()> {
    info!("tokenize: {} => {}", src_dir, dst_dir);
    let processor = ExtractedWikipediaProcessor::new()?;

    let tokenizer = VibratoTokenizer::new(system_dict.as_str(), user_dict)?;
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

    processor.write_success_file(Path::new(dst_dir))?;

    Ok(())
}

pub fn tokenize_lindera_ipadic(
    user_dict: Option<String>,
    src_dir: &str,
    dst_dir: &str,
) -> anyhow::Result<()> {
    info!("tokenize: {} => {}", src_dir, dst_dir);
    let processor = ExtractedWikipediaProcessor::new()?;

    let tokenizer = LinderaTokenizer::new(DictionaryKind::IPADIC, user_dict.map(PathBuf::from))?;
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

    processor.write_success_file(Path::new(dst_dir))?;

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
