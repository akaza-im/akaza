use std::collections::HashMap;
use std::ops::Range;



use libakaza::extend_clause::{extend_left, extend_right};
use libakaza::graph::candidate::Candidate;

use crate::input_mode::InputMode;

#[derive(Debug)]
pub struct CurrentState {
    pub(crate) input_mode: InputMode,
    pub preedit: String,
    pub(crate) clauses: Vec<Vec<Candidate>>,
    /// 現在選択されている文節
    pub(crate) current_clause: usize,
    // key は、clause 番号。value は、node の index。
    node_selected: HashMap<usize, usize>,
    /// シフト+右 or シフト+左で強制指定された範囲
    pub(crate) force_selected_clause: Vec<Range<usize>>,
}

impl CurrentState {
    pub fn new(input_mode: InputMode) -> Self {
        CurrentState {
            input_mode,
            preedit: String::new(),
            clauses: vec![],
            current_clause: 0,
            node_selected: HashMap::new(),
            force_selected_clause: Vec::new(),
        }
    }

    pub(crate) fn set_input_mode(&mut self, input_mode: &InputMode) {
        self.clear();
        self.input_mode = *input_mode;
    }

    pub fn select_candidate(&mut self, candidate_pos: usize) {
        self.node_selected
            .insert(self.current_clause, candidate_pos);
    }

    pub(crate) fn clear(&mut self) {
        self.preedit.clear();
        self.clauses.clear();

        self.clear_state();
    }

    /// 入力内容以外のものをリセットする
    pub fn clear_state(&mut self) {
        self.current_clause = 0;
        self.node_selected.clear();
        self.force_selected_clause.clear();
    }

    pub(crate) fn append_preedit(&mut self, ch: char) {
        self.preedit.push(ch);
    }

    /// バックスペースで一文字削除した場合などに呼ばれる。
    pub(crate) fn set_preedit(&mut self, preedit: String) {
        self.clear();
        self.preedit = preedit;
    }

    pub fn set_clauses(&mut self, clause: Vec<Vec<Candidate>>) {
        self.clauses = clause;
        self.node_selected.clear();
    }

    /// 変換しているときに backspace を入力した場合。
    /// 変換候補をクリアして、Conversion から Composition 状態に戻る。
    pub fn clear_clauses(&mut self) {
        self.clauses.clear();
        self.clear_state();
    }

    pub fn get_first_candidates(&self) -> Vec<Candidate> {
        let mut targets: Vec<Candidate> = Vec::new();
        for (i, candidates) in self.clauses.iter().enumerate() {
            let idx = self.node_selected.get(&i).unwrap_or(&0);
            targets.push(candidates[*idx].clone());
        }
        targets
    }

    /// 一個右の文節を選択する
    pub fn select_right_clause(&mut self) {
        if self.current_clause == self.clauses.len() - 1 {
            // 既に一番右だった場合、一番左にいく。
            self.current_clause = 0;
        } else {
            self.current_clause += 1;
        }
    }

    /// 一個左の文節を選択する
    pub fn select_left_clause(&mut self) {
        if self.current_clause == 0 {
            // 既に一番左だった場合、一番右にいく
            self.current_clause = self.clauses.len() - 1
        } else {
            self.current_clause -= 1
        }
    }

    pub fn adjust_current_clause(&mut self) {
        // [a][bc]
        //    ^^^^
        // 上記の様にフォーカスが当たっている時に extend_clause_left した場合
        // 文節の数がもとより減ることがある。その場合は index error になってしまうので、
        // current_clause を動かす。
        if self.current_clause >= self.clauses.len() {
            self.current_clause = self.clauses.len() - 1;
        }
    }

    pub fn build_string(&self) -> String {
        let mut result = String::new();
        for (clauseid, nodes) in self.clauses.iter().enumerate() {
            let idex = if let Some(i) = self.node_selected.get(&clauseid) {
                *i
            } else {
                0
            };
            if idex >= nodes.len() {
                // 発生しないはずだが、発生している。。なぜだろう?
                panic!(
                    "[BUG] self.node_selected and self.clauses missmatch: {:?}",
                    self
                )
            }
            result += &nodes[idex].surface_with_dynamic();
        }
        result
    }

    pub fn extend_right(&mut self) {
        self.force_selected_clause = extend_right(&self.clauses, self.current_clause);
    }

    pub fn extend_left(&mut self) {
        self.force_selected_clause = extend_left(&self.clauses, self.current_clause);
    }

    /// Conversion mode かどうかを判定する。
    /// Conversion mode とは space キーを一回押したあとの状態です
    pub fn in_conversion(&self) -> bool {
        !self.clauses.is_empty()
    }
}
