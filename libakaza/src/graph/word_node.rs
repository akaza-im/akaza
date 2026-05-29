use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

pub const BOS_TOKEN_KEY: &str = "__BOS__/__BOS__";
pub const EOS_TOKEN_KEY: &str = "__EOS__/__EOS__";

#[derive(Debug, Clone)]
pub struct WordNode {
    pub start_pos: i32,
    /// 表層。
    pub surface: String,
    /// 読み仮名
    pub yomi: String,
    pub cost: f32,
    pub word_id_and_score: Option<(i32, f32)>,
    pub auto_generated: bool,
    /// LM 参照用の `surface/yomi` キー。生成時に1度だけ構築してキャッシュする
    /// （Viterbi DP の最内ループで毎回 alloc するのを避けるため）。
    key: String,
}

impl Hash for WordNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start_pos.hash(state);
        self.surface.hash(state);
        self.yomi.hash(state);
        u32::from_le_bytes(self.cost.to_le_bytes()).hash(state);
    }
}

impl PartialEq<Self> for WordNode {
    fn eq(&self, other: &Self) -> bool {
        self.start_pos == other.start_pos
            && self.surface == other.surface
            && self.yomi == other.yomi
            && self.cost == other.cost
    }
}

impl Eq for WordNode {}

impl WordNode {
    /// LM 参照用の `surface/yomi` キー。生成時にキャッシュ済みの値を返す。
    pub fn key(&self) -> &str {
        &self.key
    }

    fn build_key(surface: &str, yomi: &str) -> String {
        let mut buf = String::with_capacity(surface.len() + 1 + yomi.len());
        buf.push_str(surface);
        buf.push('/');
        buf.push_str(yomi);
        buf
    }

    pub(crate) fn create_bos() -> WordNode {
        WordNode {
            start_pos: 0,
            surface: "__BOS__".to_string(),
            yomi: "__BOS__".to_string(),
            cost: 0_f32,
            word_id_and_score: None,
            auto_generated: true,
            key: BOS_TOKEN_KEY.to_string(),
        }
    }
    pub(crate) fn create_eos(start_pos: i32) -> WordNode {
        WordNode {
            start_pos,
            surface: "__EOS__".to_string(),
            yomi: "__EOS__".to_string(),
            cost: 0_f32,
            word_id_and_score: None,
            auto_generated: true,
            key: EOS_TOKEN_KEY.to_string(),
        }
    }
    pub fn new(
        start_pos: i32,
        surface: &str,
        yomi: &str,
        word_id_and_score: Option<(i32, f32)>,
        auto_generated: bool,
    ) -> WordNode {
        assert!(
            !surface.is_empty(),
            "Kanji shouldn't be empty: {surface}/{yomi}"
        );

        WordNode {
            start_pos,
            surface: surface.to_string(),
            yomi: yomi.to_string(),
            cost: 0_f32,
            word_id_and_score,
            auto_generated,
            key: Self::build_key(surface, yomi),
        }
    }
}

impl Display for WordNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.surface, self.yomi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_cached() {
        let node = WordNode::new(0, "私", "わたし", None, false);
        assert_eq!(node.key(), "私/わたし");
        // 何度呼んでも同じ値（キャッシュ済み）
        assert_eq!(node.key(), "私/わたし");
    }

    #[test]
    fn test_key_bos_eos() {
        assert_eq!(WordNode::create_bos().key(), BOS_TOKEN_KEY);
        assert_eq!(WordNode::create_eos(3).key(), EOS_TOKEN_KEY);
    }

    #[test]
    fn test_key_survives_clone() {
        let node = WordNode::new(0, "天気", "てんき", None, false);
        let cloned = node.clone();
        assert_eq!(cloned.key(), "天気/てんき");
    }
}
