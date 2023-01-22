// 加算スムージング用の定数。
const ALPHA: f32 = 0.00001;

/// 単語/エッジのコストを計算する。
/// 加算スムージングしている。
///
/// - `count`: その単語の出現回数, n(w)
/// - `total_words`: コーパス中の単語の総出現回数, `C`
/// - `unique_words`: 語彙数, `V`
pub fn calc_cost(count: u32, total_words: u32, unique_words: u32) -> f32 {
    -f32::log10(
        ((count as f32) + ALPHA) // Alpha を足す。
            / // -------
            ((total_words as f32) + ALPHA + (unique_words as f32)),
    )
}
