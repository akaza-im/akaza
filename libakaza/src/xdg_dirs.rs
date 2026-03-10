//! Cross-platform wrapper for XDG-like directory management.
//! On Unix, delegates to the `xdg` crate.
//! On non-Unix (Windows), uses `dirs` crate to provide equivalent paths.

#[cfg(unix)]
mod inner {
    pub use xdg::BaseDirectories;
}

#[cfg(not(unix))]
mod inner {
    use anyhow::Result;
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct BaseDirectories {
        prefix: String,
    }

    impl BaseDirectories {
        pub fn with_prefix(prefix: &str) -> Result<Self> {
            Ok(Self {
                prefix: prefix.to_string(),
            })
        }

        fn data_home(&self) -> PathBuf {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(&self.prefix)
        }

        fn config_home(&self) -> PathBuf {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(&self.prefix)
        }

        fn cache_home(&self) -> PathBuf {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(&self.prefix)
        }

        pub fn get_data_home(&self) -> PathBuf {
            self.data_home()
        }

        pub fn get_data_dirs(&self) -> Vec<PathBuf> {
            vec![self.data_home()]
        }

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
