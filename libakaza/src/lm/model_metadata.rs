use rsmarisa::{Agent, Keyset, Trie};

pub const BUILD_TIMESTAMP_KEY: &str = "__BUILD_TIMESTAMP__";
pub const AKAZA_DATA_VERSION_KEY: &str = "__AKAZA_DATA_VERSION__";

/// モデルファイルに埋め込むメタデータ。
#[derive(Debug, Clone, Default)]
pub struct ModelMetadata {
    pub build_timestamp: Option<String>,
    pub akaza_data_version: Option<String>,
}

impl ModelMetadata {
    /// 現在時刻と指定されたバージョン文字列でメタデータを生成する。
    pub fn now(akaza_data_version: &str) -> Self {
        ModelMetadata {
            build_timestamp: Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            akaza_data_version: Some(akaza_data_version.to_string()),
        }
    }
}

impl std::fmt::Display for ModelMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "AKAZA_DATA_VERSION: {}",
            self.akaza_data_version.as_deref().unwrap_or("(not set)")
        )?;
        writeln!(
            f,
            "BUILD_TIMESTAMP: {}",
            self.build_timestamp.as_deref().unwrap_or("(not set)")
        )?;
        Ok(())
    }
}

/// keyset にメタデータを追加する。
pub fn add_metadata_to_keyset(keyset: &mut Keyset, metadata: &ModelMetadata) {
    if let Some(ref ts) = metadata.build_timestamp {
        let key = format!("{BUILD_TIMESTAMP_KEY}\t{ts}");
        keyset.push_back_str(&key).unwrap();
    }
    if let Some(ref ver) = metadata.akaza_data_version {
        let key = format!("{AKAZA_DATA_VERSION_KEY}\t{ver}");
        keyset.push_back_str(&key).unwrap();
    }
}

/// trie からメタデータを読み取る。
pub fn read_metadata_from_trie(trie: &Trie) -> ModelMetadata {
    ModelMetadata {
        build_timestamp: read_tab_value(trie, BUILD_TIMESTAMP_KEY),
        akaza_data_version: read_tab_value(trie, AKAZA_DATA_VERSION_KEY),
    }
}

fn read_tab_value(trie: &Trie, prefix: &str) -> Option<String> {
    let query = format!("{prefix}\t");
    let mut agent = Agent::new();
    agent.set_query_str(&query);

    if trie.predictive_search(&mut agent) {
        let key = agent.key().as_str();
        if let Some((_, value)) = key.split_once('\t') {
            return Some(value.to_string());
        }
    }
    None
}
