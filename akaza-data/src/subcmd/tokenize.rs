use std::path::Path;

use log::info;
use rayon::prelude::*;

use crate::corpus_reader::aozora_bunko::AozoraBunkoProcessor;
use crate::corpus_reader::wikipedia_extracted::ExtractedWikipediaProcessor;
use crate::tokenizer::base::AkazaTokenizer;
use crate::tokenizer::vibrato::VibratoTokenizer;

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
