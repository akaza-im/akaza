use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;

pub(crate) fn read_user_stats_file(path: &String) -> Result<Vec<(String, u32)>> {
    let file = File::open(path)?;

    let mut result: Vec<(String, u32)> = Vec::new();

    for line in BufReader::new(file).lines() {
        let line = line.context("Cannot read user language model file")?;
        let Some((key, count)) = line.trim().split_once(' ') else {
            continue;
        };

        let count = count
            .to_string()
            .parse::<u32>()
            .with_context(|| format!("Invalid line in user language model: {}", count))?;

        result.push((key.to_string(), count));
    }

    Ok(result)
}

pub(crate) fn write_user_stats_file(path: &str, word_count: &HashMap<String, u32>) -> Result<()> {
    let mut tmpfile = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path.to_string() + ".tmp")?;

    for (key, cnt) in word_count {
        tmpfile.write_all(key.as_bytes())?;
        tmpfile.write_all(" ".as_bytes())?;
        tmpfile.write_all(cnt.to_string().as_bytes())?;
        tmpfile.write_all("\n".as_bytes())?;
    }
    fs::rename(path.to_owned() + ".tmp", path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();
        write_user_stats_file(&path, &HashMap::from([("渡し".to_string(), 3_u32)])).unwrap();
        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "渡し 3\n");
    }
}
