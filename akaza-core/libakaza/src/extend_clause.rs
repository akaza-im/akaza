use std::collections::VecDeque;
use std::ops::Range;

use crate::graph::graph_resolver::Candidate;

/// 文節の選択範囲を右に拡張する。
/// current_clause は現在選択されている分節。左から 0 origin である。
pub fn extend_right(
    clauses: &Vec<VecDeque<Candidate>>,
    current_clause: usize,
) -> Vec<Range<usize>> {
    // カラだったらなにもできない。
    if clauses.is_empty() {
        return Vec::new();
    }
    // 一番右の文節が選択されていたらなにもできない。
    if current_clause == clauses.len() - 1 {
        return Vec::new();
    }

    // Note: Rust の range は exclusive.

    let mut force_selected_clause: Vec<Range<usize>> = Vec::new();
    let mut offset = 0;
    for (i, clause) in clauses.iter().enumerate() {
        let candidate = &clause[0];
        if current_clause == i {
            // 現在選択中の文節は、右に伸ばす
            let next_candidate = &clauses[i + 1][0];
            force_selected_clause.push(
                offset
                    ..offset
                        + candidate.yomi.len()
                        + next_candidate.yomi.chars().next().unwrap().len_utf8(),
            );
        } else if current_clause + 1 == i {
            // 選択中の分節の右のものは、1文字減らされる。
            let c = candidate.yomi.chars().next().unwrap();
            let first_char_len = c.len_utf8();
            let start = offset + first_char_len;
            let end = offset + first_char_len + candidate.yomi.len() - first_char_len;
            if start < end {
                // 前の文節を拡張した結果、次の文節がなくなるケースもある。
                force_selected_clause.push(start..end);
            }
        } else {
            force_selected_clause.push(offset..offset + candidate.yomi.len())
        }

        offset += candidate.yomi.len();
    }
    force_selected_clause
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extend_right() {
        let clauses = vec![VecDeque::from([Candidate::new("わ", "わ", 0_f32)])];
        assert_eq!(extend_right(clauses, 0), vec!(), "Only 1 clause");
    }

    // 第1文節を拡張した結果、第2文節がなくなるケース
    #[test]
    fn test_extend_right2() {
        let clauses = vec![
            VecDeque::from([Candidate::new("わ", "わ", 0_f32)]),
            VecDeque::from([Candidate::new("た", "た", 0_f32)]),
        ];
        let got = extend_right(clauses, 0);
        assert_ne!(got, vec!(), "There's 2 clause");
        let g1 = got[0].clone();
        let p = &("わた".to_string()[g1]);
        assert_eq!(p, "わた");
        assert_eq!(got.len(), 1);
    }

    // ちゃんと伸ばせるケース
    #[test]
    fn test_extend_right3() {
        let clauses = vec![
            VecDeque::from([Candidate::new("わ", "わ", 0_f32)]),
            VecDeque::from([Candidate::new("たし", "たし", 0_f32)]),
        ];
        let got = extend_right(clauses, 0);
        assert_ne!(got, vec!(), "There's 2 clause");
        assert_eq!(got.len(), 2);
        assert_eq!(&("わたし".to_string()[got[0].clone()]), "わた");
        assert_eq!(&("わたし".to_string()[got[1].clone()]), "し");
    }
}
