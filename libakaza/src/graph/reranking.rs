use serde::{Deserialize, Serialize};

use super::graph_resolver::KBestPath;

/// リランキング重み。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReRankingWeights {
    // unigram_weight は 1.0 固定（基準スケール）
    /// 既知 bigram コストの重み（デフォルト 1.0）
    pub bigram_weight: f32,
    /// トークン長ペナルティの重み（デフォルト 2.0）
    pub length_weight: f32,
    /// 未知 bigram フォールバックコストの重み（デフォルト 1.0）
    pub unknown_bigram_weight: f32,
    /// skip-bigram コストの重み（デフォルト 0.0 = 無効）
    #[serde(default)]
    pub skip_bigram_weight: f32,
}

impl Default for ReRankingWeights {
    fn default() -> Self {
        ReRankingWeights {
            bigram_weight: 1.0,
            length_weight: 2.0,
            unknown_bigram_weight: 1.0,
            skip_bigram_weight: 0.2,
        }
    }
}

impl ReRankingWeights {
    /// パスの rerank_cost を再計算し、スコア昇順にソートする。
    pub fn rerank(&self, paths: &mut [KBestPath]) {
        for path in paths.iter_mut() {
            path.rerank_cost = path.unigram_cost
                + self.bigram_weight * path.bigram_cost
                + self.unknown_bigram_weight * path.unknown_bigram_cost
                + self.length_weight * path.token_count as f32
                + self.skip_bigram_weight * path.skip_bigram_cost;
        }
        paths.sort_by(|a, b| a.rerank_cost.partial_cmp(&b.rerank_cost).unwrap());
    }

    /// デフォルト重みかどうか
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_path(
        viterbi_cost: f32,
        unigram_cost: f32,
        bigram_cost: f32,
        unknown_bigram_cost: f32,
        unknown_bigram_count: u32,
        token_count: u32,
    ) -> KBestPath {
        KBestPath {
            segments: Vec::new(),
            cost: viterbi_cost,
            viterbi_cost,
            unigram_cost,
            bigram_cost,
            unknown_bigram_cost,
            unknown_bigram_count,
            token_count,
            rerank_cost: viterbi_cost,
            word_ids: Vec::new(),
            skip_bigram_cost: 0.0,
        }
    }

    #[test]
    fn test_default_weights_rerank_cost() {
        let weights = ReRankingWeights::default();
        let mut paths = vec![
            make_path(10.0, 3.0, 2.0, 5.0, 1, 3),
            make_path(8.0, 4.0, 1.0, 3.0, 1, 2),
        ];
        weights.rerank(&mut paths);

        // デフォルト重みでは rerank_cost == unigram + 1.0*bigram + 1.0*unknown + 2.0*token
        for path in &paths {
            let expected = path.unigram_cost
                + path.bigram_cost
                + path.unknown_bigram_cost
                + 2.0 * path.token_count as f32;
            assert!(
                (path.rerank_cost - expected).abs() < f32::EPSILON,
                "rerank_cost={} expected={}",
                path.rerank_cost,
                expected
            );
        }
    }

    #[test]
    fn test_custom_weights_change_ranking() {
        let weights = ReRankingWeights {
            bigram_weight: 0.5,
            length_weight: 0.0,
            unknown_bigram_weight: 0.1,
            skip_bigram_weight: 0.0,
        };

        // path A: unigram=3, bigram=2, unknown=10 → 3 + 0.5*2 + 0.1*10 = 5.0
        // path B: unigram=5, bigram=1, unknown=0  → 5 + 0.5*1 + 0.1*0  = 5.5
        let mut paths = vec![
            make_path(15.0, 3.0, 2.0, 10.0, 2, 3),
            make_path(6.0, 5.0, 1.0, 0.0, 0, 2),
        ];
        weights.rerank(&mut paths);

        assert!((paths[0].rerank_cost - 5.0).abs() < f32::EPSILON);
        assert!((paths[1].rerank_cost - 5.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_length_weight() {
        let weights = ReRankingWeights {
            bigram_weight: 1.0,
            length_weight: 2.0,
            unknown_bigram_weight: 1.0,
            skip_bigram_weight: 0.0,
        };

        // path A: unigram=3, bigram=2, unknown=1, tokens=5 → 3+2+1+2*5 = 16
        // path B: unigram=3, bigram=2, unknown=1, tokens=2 → 3+2+1+2*2 = 10
        let mut paths = vec![
            make_path(6.0, 3.0, 2.0, 1.0, 1, 5),
            make_path(6.0, 3.0, 2.0, 1.0, 1, 2),
        ];
        weights.rerank(&mut paths);

        assert!((paths[0].rerank_cost - 10.0).abs() < f32::EPSILON);
        assert!((paths[1].rerank_cost - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_default() {
        assert!(ReRankingWeights::default().is_default());
        assert!(!ReRankingWeights {
            bigram_weight: 0.5,
            length_weight: 0.0,
            unknown_bigram_weight: 1.0,
            skip_bigram_weight: 0.0,
        }
        .is_default());
    }
}
