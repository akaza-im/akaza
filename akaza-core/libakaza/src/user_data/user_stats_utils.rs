use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::{fs, io};

pub(crate) fn read_user_stats_file(path: &String) -> Result<Vec<(String, u32)>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut result: Vec<(String, u32)> = Vec::new();

    for line in BufReader::new(file).lines() {
        let Ok(line) = line else {
            return Err("Cannot read user language model file".to_string());
        };
        let Some((key, count)) = line.trim().split_once(" ") else {
            continue;
        };

        let Ok(count) = count.to_string().parse::<u32>() else {
            return Err("Invalid line in user language model: ".to_string() + count);
        };

        result.push((key.to_string(), count));
    }

    return Ok(result);
}

pub(crate) fn write_user_stats_file(
    path: &String,
    word_count: &HashMap<String, u32>,
) -> Result<(), io::Error> {
    let mut tmpfile = File::create(path.clone() + ".tmp")?;

    for (key, cnt) in word_count {
        tmpfile.write(key.as_bytes())?;
        tmpfile.write(" ".as_bytes())?;
        tmpfile.write(cnt.to_string().as_bytes())?;
        tmpfile.write("\n".as_bytes())?;
    }
    fs::rename(path.clone() + ".tmp", path.clone())?;

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
