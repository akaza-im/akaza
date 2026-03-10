//! Cross-platform wrapper for XDG-like directory management.
//! On Unix, delegates to the `xdg` crate.
//! On non-Unix (Windows), uses `dirs` crate to provide equivalent paths.

#[cfg(unix)]
mod inner {
    pub use xdg::BaseDirectories;
}

#[cfg(not(unix))]
mod inner {
    use anyhow::{bail, Result};
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct BaseDirectories {
        prefix: String,
        data_home: PathBuf,
        config_home: PathBuf,
        cache_home: PathBuf,
    }

    impl BaseDirectories {
        pub fn with_prefix(prefix: &str) -> Result<Self> {
            let data_home = dirs::data_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?
                .join(prefix);
            let config_home = dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?
                .join(prefix);
            let cache_home = dirs::cache_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine cache directory"))?
                .join(prefix);
            Ok(Self {
                prefix: prefix.to_string(),
                data_home,
                config_home,
                cache_home,
            })
        }

        fn data_home(&self) -> &Path {
            &self.data_home
        }

        fn config_home(&self) -> &Path {
            &self.config_home
        }

        fn cache_home(&self) -> &Path {
            &self.cache_home
        }

        pub fn get_data_home(&self) -> PathBuf {
            self.data_home().to_path_buf()
        }

        pub fn get_data_dirs(&self) -> Vec<PathBuf> {
            vec![self.data_home().to_path_buf()]
        }

        /// NOTE: Unix の xdg crate とは異なり、data_home のみを検索する。
        /// XDG_DATA_DIRS 相当の追加検索パスは未実装。
        pub fn find_data_file<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
            let full = self.data_home().join(path.as_ref());
            if full.exists() {
                Some(full)
            } else {
                None
            }
        }

        pub fn find_data_files<P: AsRef<Path>>(&self, path: P) -> impl Iterator<Item = PathBuf> {
            self.find_data_file(path).into_iter()
        }

        pub fn place_data_file<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
            let full = self.data_home().join(path.as_ref());
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent)?;
            }
            Ok(full)
        }

        /// NOTE: Unix の xdg crate の get_config_file とは異なり、
        /// ファイルの存在チェックを行わず常にパスを返す。
        pub fn get_config_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
            self.config_home().join(path.as_ref())
        }

        pub fn create_cache_directory<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
            let full = self.cache_home().join(path.as_ref());
            fs::create_dir_all(&full)?;
            Ok(full)
        }

        pub fn get_cache_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
            self.cache_home().join(path.as_ref())
        }
    }
}

pub use inner::BaseDirectories;
