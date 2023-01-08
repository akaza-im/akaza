use std::path::Path;

use log::info;

use crate::tokenizer::vibrato::VibratoTokenizer;
use crate::wikipedia::wikipedia_extracted::ExtractedWikipediaProcessor;

pub fn annotate_wikipedia(src_dir: &str, dst_dir: &str) -> anyhow::Result<()> {
    info!("annotate_wikipedia: {} => {}", src_dir, dst_dir);
    let runner = VibratoTokenizer::new()?;
    info!("Initialized tokenizer");

    let processor = ExtractedWikipediaProcessor::new()?;
    processor.process_files(Path::new(src_dir), Path::new(dst_dir), |line| {
        runner.tokenize(line)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    #[ignore]
    fn test_wikipedia() -> anyhow::Result<()> {
        let runner = VibratoTokenizer::new()?;
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

    #[test]
    #[ignore]
    fn test_all() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        annotate_wikipedia()?;
        Ok(())
    }
}
