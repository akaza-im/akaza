use std::collections::HashMap;
use std::ops::Range;

use ibus_sys::attr_list::{ibus_attr_list_append, ibus_attr_list_new};
use ibus_sys::attribute::{
    ibus_attribute_new, IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE, IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
};
use ibus_sys::core::to_gboolean;
use ibus_sys::engine::{ibus_engine_update_preedit_text, IBusEngine};
use ibus_sys::glib::guint;
use ibus_sys::text::{ibus_text_set_attributes, StringExt};
use libakaza::extend_clause::{extend_left, extend_right};
use libakaza::graph::candidate::Candidate;

use crate::input_mode::InputMode;

#[derive(Debug)]
pub struct CurrentState {
    pub(crate) input_mode: InputMode,
    preedit: String,
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

    pub(crate) fn set_input_mode(&mut self, engine: *mut IBusEngine, input_mode: &InputMode) {
        self.clear(engine);
        self.input_mode = *input_mode;
    }

    pub fn select_candidate(&mut self, candidate_pos: usize) {
        self.node_selected
            .insert(self.current_clause, candidate_pos);
    }

    pub(crate) fn clear(&mut self, engine: *mut IBusEngine) {
        self.preedit.clear();
        self.on_preedit_change(engine);
        self.clauses.clear();

        self.clear_state();
    }

    pub fn get_preedit(&self) -> &str {
        &self.preedit
    }

    /// 入力内容以外のものをリセットする
    pub fn clear_state(&mut self) {
        self.current_clause = 0;
        self.node_selected.clear();
        self.force_selected_clause.clear();
    }

    pub(crate) fn append_preedit(&mut self, engine: *mut IBusEngine, ch: char) {
        self.preedit.push(ch);
        self.on_preedit_change(engine);
    }

    /// バックスペースで一文字削除した場合などに呼ばれる。
    pub(crate) fn set_preedit(&mut self, engine: *mut IBusEngine, preedit: String) {
        self.preedit = preedit;
        self.clauses.clear();
        self.clear_state();
        self.on_preedit_change(engine);
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
                panic!("[BUG] self.node_selected and self.clauses missmatch")
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

    pub fn on_preedit_change(&self, engine: *mut IBusEngine) {
        unsafe {
            let current_clause = &self.clauses[self.current_clause];
            let current_node = &(current_clause[0]);

            let text = self.build_string();
            let preedit_attrs = ibus_attr_list_new();
            // 全部に下線をひく。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
                    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
                    0,
                    text.len() as guint,
                ),
            );
            let bgstart: u32 = self
                .clauses
                .iter()
                .map(|c| (c[0].surface).len() as u32)
                .sum();
            // 背景色を設定する。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
                    0x00333333,
                    bgstart,
                    bgstart + (current_node.surface.len() as u32),
                ),
            );
            let preedit_text = text.to_ibus_text();
            ibus_text_set_attributes(preedit_text, preedit_attrs);
            ibus_engine_update_preedit_text(
                engine,
                preedit_text,
                text.len() as guint,
                to_gboolean(!text.is_empty()),
            );
        }
    }
}
