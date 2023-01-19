use std::fs;
use std::path::Path;

use anyhow::bail;
use log::info;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::corpus_reader::aozora_bunko::AozoraBunkoProcessor;
use crate::corpus_reader::base::{write_success_file, CorpusReader};
use crate::corpus_reader::wikipedia_extracted::ExtractedWikipediaProcessor;
use crate::tokenizer::base::AkazaTokenizer;
use crate::tokenizer::vibrato::VibratoTokenizer;

pub fn tokenize(
    reader: String,
    system_dict: String,
    user_dict: Option<String>,
    src_dir: &str,
    dst_dir: &str,
) -> anyhow::Result<()> {
    info!("tokenize: {} => {}", src_dir, dst_dir);

    let tokenizer = VibratoTokenizer::new(system_dict.as_str(), user_dict)?;
    let file_list = get_file_list(Path::new(src_dir), Path::new(dst_dir))?;

    match reader.as_str() {
        "jawiki" => {
            let processor = ExtractedWikipediaProcessor::new()?;
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
        "aozora_bunko" => {
            let processor = AozoraBunkoProcessor::new()?;
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
        _ => bail!("Unknown reader :{}", reader),
    }

    write_success_file(Path::new(dst_dir))?;

    Ok(())
}

fn get_file_list(src_dir: &Path, dst_dir: &Path) -> anyhow::Result<Vec<(String, String)>> {
    let mut result: Vec<(String, String)> = Vec::new();

    for src_file in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.metadata().unwrap().is_file())
    {
        let src_path = src_file.path();
        let dirname = src_path.parent().unwrap().file_name().unwrap();
        fs::create_dir_all(dst_dir.join(dirname))?;
        let output_file = dst_dir.join(dirname).join(src_path.file_name().unwrap());

        result.push((
            src_file.path().to_string_lossy().to_string(),
            output_file.as_path().to_string_lossy().to_string(),
        ));
    }
    Ok(result)
}
