// 加算スムージング用の定数。
const ALPHA: f32 = 0.00001;

// 確率の計算。
// 加算スムージングをかけている。
pub fn calc_cost(count: u32, unique_words: u32, total_words: u32) -> f32 {
    f32::log10(((count as f32) + ALPHA) / ((unique_words as f32) + ALPHA + (total_words as f32)))
}
