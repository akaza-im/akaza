use std::fs::{self, OpenOptions};
use std::path::Path;

use anyhow::Result;
use log::{info, warn};
use rand::RngCore;

const KEY_LEN: usize = 32;

/// 暗号化鍵を読み込む。なければ生成して保存する。
///
/// 鍵ファイルは 0600 パーミッションで保存される。
pub fn load_or_create_encryption_key(key_path: &Path) -> Result<Vec<u8>> {
    if key_path.exists() {
        let key = fs::read(key_path)?;
        if key.len() == KEY_LEN {
            info!("Loaded encryption key from {}", key_path.display());
            return Ok(key);
        }
        warn!(
            "Invalid encryption key size ({} bytes), regenerating",
            key.len()
        );
    }

    let mut key = vec![0u8; KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);

    // 親ディレクトリがなければ作成
    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(key_path)?;
        file.write_all(&key)?;
    }
    #[cfg(not(unix))]
    {
        use std::io::Write;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(key_path)?;
        file.write_all(&key)?;
    }

    info!("Generated new encryption key at {}", key_path.display());
    Ok(key)
}

/// デフォルトの鍵ファイルパスから暗号化鍵を読み込む。なければ生成する。
pub fn load_or_create_default_encryption_key() -> Result<Vec<u8>> {
    let basedir = xdg::BaseDirectories::with_prefix("akaza")?;
    let key_path = basedir.place_data_file(Path::new("encryption.key"))?;
    load_or_create_encryption_key(&key_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_load_key() {
        let tmpdir = TempDir::new().unwrap();
        let key_path = tmpdir.path().join("encryption.key");

        // 初回: 鍵を生成
        let key1 = load_or_create_encryption_key(&key_path).unwrap();
        assert_eq!(key1.len(), KEY_LEN);

        // 再度: 同じ鍵が読み込まれる
        let key2 = load_or_create_encryption_key(&key_path).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_regenerate_invalid_key() {
        let tmpdir = TempDir::new().unwrap();
        let key_path = tmpdir.path().join("encryption.key");

        // 不正な長さの鍵ファイルを書き込む
        fs::write(&key_path, b"short").unwrap();

        // 再生成される
        let key = load_or_create_encryption_key(&key_path).unwrap();
        assert_eq!(key.len(), KEY_LEN);
        assert_ne!(key, b"short");
    }
}
