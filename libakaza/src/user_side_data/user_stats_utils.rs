use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::{bail, Context, Result};
use rand::RngCore;
use rustc_hash::FxHashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// Magic bytes for v2 encrypted format: "AKZ\x01"
const V2_MAGIC: &[u8; 4] = b"AKZ\x01";
const IV_LEN: usize = 16;

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
            .with_context(|| format!("Invalid line in user language model: {count}"))?;

        result.push((key.to_string(), count));
    }

    Ok(result)
}

pub(crate) fn write_user_stats_file(path: &str, word_count: &FxHashMap<String, u32>) -> Result<()> {
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);
    let mut tmpfile = opts.open(path.to_string() + ".tmp")?;

    for (key, cnt) in word_count {
        tmpfile.write_all(key.as_bytes())?;
        tmpfile.write_all(" ".as_bytes())?;
        tmpfile.write_all(cnt.to_string().as_bytes())?;
        tmpfile.write_all("\n".as_bytes())?;
    }
    fs::rename(path.to_owned() + ".tmp", path)?;

    Ok(())
}

/// Read v2 encrypted binary file.
/// Format: [magic 4B: "AKZ\x01"][IV 16B][encrypted bincode(Vec<(String, u32)>)]
pub(crate) fn read_user_stats_file_v2(path: &str, key: &[u8]) -> Result<Vec<(String, u32)>> {
    if key.len() != 32 {
        bail!("encryption key must be 32 bytes, got {} bytes", key.len());
    }
    let data = fs::read(path)?;
    if data.len() < V2_MAGIC.len() + IV_LEN {
        bail!("v2 file too short: {}", path);
    }
    if &data[..4] != V2_MAGIC {
        bail!("Invalid v2 magic in: {}", path);
    }
    let iv = &data[4..4 + IV_LEN];
    let ciphertext = &data[4 + IV_LEN..];

    let decrypted = Aes256CbcDec::new_from_slices(key, iv)
        .map_err(|e| anyhow::anyhow!("AES init error: {}", e))?
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| anyhow::anyhow!("AES decrypt error: {}", e))?;

    let result: Vec<(String, u32)> =
        bincode::deserialize(&decrypted).context("Failed to deserialize v2 user stats")?;
    Ok(result)
}

/// Write v2 encrypted binary file.
/// Format: [magic 4B: "AKZ\x01"][IV 16B][encrypted bincode(Vec<(String, u32)>)]
pub(crate) fn write_user_stats_file_v2(
    path: &str,
    key: &[u8],
    word_count: &FxHashMap<String, u32>,
) -> Result<()> {
    if key.len() != 32 {
        bail!("encryption key must be 32 bytes, got {} bytes", key.len());
    }
    let entries: Vec<(String, u32)> = word_count.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let plaintext = bincode::serialize(&entries)?;

    let mut iv = [0u8; IV_LEN];
    rand::thread_rng().fill_bytes(&mut iv);

    let ciphertext = Aes256CbcEnc::new_from_slices(key, &iv)
        .map_err(|e| anyhow::anyhow!("AES init error: {}", e))?
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let tmp_path = path.to_owned() + ".tmp";
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);
    let mut tmpfile = opts.open(&tmp_path)?;

    tmpfile.write_all(V2_MAGIC)?;
    tmpfile.write_all(&iv)?;
    tmpfile.write_all(&ciphertext)?;
    fs::rename(&tmp_path, path)?;

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
        let wc: FxHashMap<String, u32> = [("渡し".to_string(), 3_u32)].into_iter().collect();
        write_user_stats_file(&path, &wc).unwrap();
        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "渡し 3\n");
    }

    #[test]
    fn test_v2_roundtrip() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();

        let key = [0x42u8; 32]; // 256-bit key
        let mut wc: FxHashMap<String, u32> = FxHashMap::default();
        wc.insert("渡し/わたし".to_string(), 3);
        wc.insert("互換/ごかん".to_string(), 5);

        write_user_stats_file_v2(&path, &key, &wc).unwrap();
        let result = read_user_stats_file_v2(&path, &key).unwrap();

        let result_map: FxHashMap<String, u32> = result.into_iter().collect();
        assert_eq!(result_map, wc);
    }

    #[test]
    fn test_v2_not_readable_as_text() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();

        let key = [0x42u8; 32];
        let mut wc: FxHashMap<String, u32> = FxHashMap::default();
        wc.insert("渡し/わたし".to_string(), 3);

        write_user_stats_file_v2(&path, &key, &wc).unwrap();

        // v2 ファイルがテキストとして読めないことを確認
        let data = fs::read(&path).unwrap();
        assert_eq!(&data[..4], V2_MAGIC);
        // テキスト形式の v1 ではないことを確認
        let text_result = String::from_utf8(data);
        assert!(
            text_result.is_err() || !text_result.unwrap().contains("渡し"),
            "Encrypted file should not contain plaintext"
        );
    }

    #[test]
    fn test_v2_wrong_key_fails() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();

        let key = [0x42u8; 32];
        let wrong_key = [0x43u8; 32];
        let mut wc: FxHashMap<String, u32> = FxHashMap::default();
        wc.insert("渡し/わたし".to_string(), 3);

        write_user_stats_file_v2(&path, &key, &wc).unwrap();
        let result = read_user_stats_file_v2(&path, &wrong_key);
        assert!(result.is_err(), "Wrong key should fail");
    }

    #[test]
    fn test_v2_empty_hashmap() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();

        let key = [0x42u8; 32];
        let wc: FxHashMap<String, u32> = FxHashMap::default();

        write_user_stats_file_v2(&path, &key, &wc).unwrap();
        let result = read_user_stats_file_v2(&path, &key).unwrap();
        assert!(result.is_empty(), "Empty hashmap should roundtrip as empty");
    }

    #[test]
    fn test_v2_large_dataset() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();

        let key = [0x42u8; 32];
        let mut wc: FxHashMap<String, u32> = FxHashMap::default();
        for i in 0..10_000 {
            wc.insert(format!("単語{}/たんご{}", i, i), i as u32);
        }

        write_user_stats_file_v2(&path, &key, &wc).unwrap();
        let result = read_user_stats_file_v2(&path, &key).unwrap();

        let result_map: FxHashMap<String, u32> = result.into_iter().collect();
        assert_eq!(result_map.len(), 10_000);
        assert_eq!(result_map, wc);
    }

    #[test]
    fn test_v2_invalid_key_length() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_str().unwrap().to_string();

        let short_key = [0x42u8; 16];
        let wc: FxHashMap<String, u32> = FxHashMap::default();

        let result = write_user_stats_file_v2(&path, &short_key, &wc);
        assert!(result.is_err(), "Short key should be rejected");
    }
}
