use std::collections::vec_deque::VecDeque;
use std::ops::Range;

use crate::graph::graph_resolver::Candidate;

// 現状維持するための文節データを返します。
fn keep_current(clauses: &[VecDeque<Candidate>]) -> Vec<Range<usize>> {
    let mut force_selected_clause: Vec<Range<usize>> = Vec::new();
    let mut offset = 0;
    for yomi_len in clauses.iter().map(|f| f[0].yomi.len()) {
        force_selected_clause.push(offset..offset + yomi_len);
        offset += yomi_len;
    }
    force_selected_clause
}

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
        return keep_current(clauses);
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

/// 文節の選択範囲を **左** に拡張する。
/// current_clause は現在選択されている分節。左から 0 origin である。
pub fn extend_left(clauses: &Vec<VecDeque<Candidate>>, current_clause: usize) -> Vec<Range<usize>> {
    if clauses.is_empty() {
        return Vec::new();
    }
    if clauses.len() == 1 {
        // 分節が一個の場合
        let yomi = &clauses[0][0].yomi;
        return if yomi.chars().count() > 1 {
            // 最後の文字を別分節に切り出す。
            let mut force_selected_clause: Vec<Range<usize>> = Vec::new();
            let last_char = yomi.chars().last().unwrap();
            force_selected_clause.push(0..yomi.len() - last_char.len_utf8());
            force_selected_clause.push(yomi.len() - last_char.len_utf8()..yomi.len());
            force_selected_clause
        } else {
            // Only 1 character.
            keep_current(clauses)
        };
    }

    if current_clause == 0 {
        // 一番左の文節にフォーカスがあたっているので、一番左の分節を短くする。

        if clauses[0][0].yomi.chars().count() == 1 {
            // 一番左の分節が1文字しかないときは現状維持の形で返す。
            return keep_current(clauses);
        }

        let mut force_selected_clause: Vec<Range<usize>> = Vec::new();
        let mut offset = 0;
        for (i, clause) in clauses.iter().enumerate() {
            // AS-IS: [ab][c]
            //         ^^ <- focused
            //
            // TO-BE: [a][bc]
            let yomi = &clause[0].yomi;
            if i == current_clause {
                let last_char = yomi.chars().last().unwrap();
                force_selected_clause.push(offset..offset + yomi.len() - last_char.len_utf8());
            } else if i == current_clause + 1 {
                let prev_last_char = clauses[i - 1][0].yomi.chars().last().unwrap().len_utf8();
                let start = offset - prev_last_char;
                let end = start + (yomi.len() + prev_last_char);
                // 消失するケースもある
                if start < end {
                    force_selected_clause.push(start..end);
                }
            } else {
                force_selected_clause.push(offset..offset + yomi.len());
            }
            offset += yomi.len();
        }
        force_selected_clause
    } else {
        // ニ番目以後の分節にフォーカスがあたっているので、左隣の分節を短くし、フォーカスがあたっている分節を伸ばします。
        let mut force_selected_clause: Vec<Range<usize>> = Vec::new();
        let mut offset = 0;
        for (i, clause) in clauses.iter().enumerate() {
            let yomi = &clause[0].yomi;
            let (start, end) = if i == current_clause {
                let prev_yomi = &clauses[i - 1][0].yomi;
                let prev_last_char = prev_yomi.chars().last().unwrap().len_utf8();
                let start = offset - prev_last_char;
                let end = start + yomi.len() + prev_last_char;
                (start, end)
            } else if i == current_clause - 1 {
                // フォーカス文節の左の文節は、末尾の文字を対象から外す
                let last_char = yomi.chars().last().unwrap().len_utf8();
                let start = offset;
                let end = offset + (yomi.len() - last_char);
                // 消失するケースもある
                (start, end)
            } else {
                let start = offset;
                let end = offset + yomi.len();
                (start, end)
            };
            if start < end {
                force_selected_clause.push(start..end);
            }
            offset += yomi.len();
        }
        force_selected_clause
    }
}

#[cfg(test)]
mod test_base {
    use super::*;

    pub fn mk(src: &[&str]) -> (String, Vec<VecDeque<Candidate>>) {
        let mut clauses: Vec<VecDeque<Candidate>> = Vec::new();
        for x in src {
            clauses.push(VecDeque::from([Candidate::new(x, x, 0_f32)]));
        }
        let yomi = src.join("");
        (yomi, clauses)
    }

    pub fn to_vec(yomi: String, got: Vec<Range<usize>>) -> Vec<String> {
        got.iter().map(|it| yomi[it.clone()].to_string()).collect()
    }
}

#[cfg(test)]
mod tests_right {
    use super::test_base::mk;
    use super::test_base::to_vec;
    use super::*;

    #[test]
    fn test_extend_right() {
        let (yomi, clauses) = mk(&["わ"]);
        let got = extend_right(&clauses, 0);
        assert_eq!(to_vec(yomi, got), vec!("わ"));
    }

    // 第1文節を拡張した結果、第2文節がなくなるケース
    #[test]
    fn test_extend_right2() {
        let (yomi, clauses) = mk(&["わ", "た"]);
        let got = extend_right(&clauses, 0);
        assert_eq!(to_vec(yomi, got), vec!("わた"));
    }

    // ちゃんと伸ばせるケース
    #[test]
    fn test_extend_right3() {
        let (yomi, clauses) = mk(&["わ", "たし"]);
        let got = extend_right(&clauses, 0);
        assert_eq!(to_vec(yomi, got), vec!("わた", "し"));
    }
}

#[cfg(test)]
mod tests_left {
    use super::test_base::mk;
    use super::test_base::to_vec;
    use super::*;

    #[test]
    fn test_extend_left() {
        let (yomi, clauses) = mk(&["わ"]);
        let got = extend_left(&clauses, 0);
        assert_eq!(to_vec(yomi, got), vec!("わ"));
    }

    // 第1文節が選択されていて、第1文節が1文字のケース
    #[test]
    fn test_extend_left2() {
        let (yomi, clauses) = mk(&["わ", "た"]);
        let got = extend_left(&clauses, 0);
        assert_eq!(to_vec(yomi, got), vec!("わ", "た"));
    }

    // 第1文節が選択されていて、第1文節が2文字以上のケース
    #[test]
    fn test_extend_left3() {
        let (yomi, clauses) = mk(&["わだ", "た", "そ"]);
        let got = extend_left(&clauses, 0);
        assert_eq!(to_vec(yomi, got), vec!("わ", "だた", "そ"));
    }

    // 第2文節が選択されている
    #[test]
    fn test_extend_left4() {
        let (yomi, clauses) = mk(&["わだ", "た", "そ"]);
        let got = extend_left(&clauses, 1);
        assert_eq!(to_vec(yomi, got), vec!("わ", "だた", "そ"));
    }

    // 文節が追加されるべき
    #[test]
    fn test_extend_left5() {
        let (yomi, clauses) = mk(&["およよよあ"]);
        let got = extend_left(&clauses, 0);
        assert_eq!(to_vec(yomi, got), vec!("およよよ", "あ"));
    }

    // 文節がマージされるべき
    #[test]
    fn test_extend_left6() {
        let (yomi, clauses) = mk(&["や", "まと"]);
        let got = extend_left(&clauses, 1);
        assert_eq!(to_vec(yomi, got), vec!("やまと"));
    }
}
