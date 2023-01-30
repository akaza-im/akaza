use std::collections::HashMap;
use std::ops::Range;

use log::info;

use ibus_sys::attr_list::{ibus_attr_list_append, ibus_attr_list_new};
use ibus_sys::attribute::{
    ibus_attribute_new, IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE, IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
};
use ibus_sys::core::to_gboolean;
use ibus_sys::engine::{
    ibus_engine_hide_auxiliary_text, ibus_engine_hide_preedit_text,
    ibus_engine_update_auxiliary_text, ibus_engine_update_lookup_table,
    ibus_engine_update_preedit_text, IBusEngine,
};
use ibus_sys::glib::guint;
use ibus_sys::lookup_table::IBusLookupTable;
use ibus_sys::text::{ibus_text_set_attributes, StringExt};
use libakaza::extend_clause::{extend_left, extend_right};
use libakaza::graph::candidate::Candidate;
use libakaza::keymap::KeyState;

use crate::input_mode::InputMode;

#[derive(Debug)]
pub struct CurrentState {
    pub(crate) input_mode: InputMode,
    raw_input: String,
    auxiliary_text: String,
    pub(crate) clauses: Vec<Vec<Candidate>>,
    /// 現在選択されている文節
    pub(crate) current_clause: usize,
    // key は、clause 番号。value は、node の index。
    node_selected: HashMap<usize, usize>,
    /// シフト+右 or シフト+左で強制指定された範囲
    pub(crate) force_selected_clause: Vec<Range<usize>>,
    /// ライブコンバージョン
    pub live_conversion: bool,
    pub(crate) lookup_table_visible: bool,
    pub lookup_table: IBusLookupTable,
}

impl CurrentState {
    pub fn new(input_mode: InputMode, live_conversion: bool) -> Self {
        CurrentState {
            input_mode,
            raw_input: String::new(),
            auxiliary_text: String::new(),
            clauses: vec![],
            current_clause: 0,
            node_selected: HashMap::new(),
            force_selected_clause: Vec::new(),
            live_conversion,
            lookup_table_visible: false,
            lookup_table: IBusLookupTable::new(10, 0, 1, 1),
        }
    }

    pub(crate) fn set_input_mode(&mut self, engine: *mut IBusEngine, input_mode: &InputMode) {
        self.clear(engine);
        self.input_mode = *input_mode;
    }

    pub fn select_candidate(&mut self, engine: *mut IBusEngine, candidate_pos: usize) {
        self.node_selected
            .insert(self.current_clause, candidate_pos);
        self.render_preedit(engine);
    }

    pub(crate) fn clear(&mut self, engine: *mut IBusEngine) {
        if !self.raw_input.is_empty() {
            self.raw_input.clear();
            self.on_raw_input_change(engine);
        }

        self.clear_clauses(engine);

        self.clear_state(engine);
    }

    pub fn get_raw_input(&self) -> &str {
        &self.raw_input
    }

    /// 入力内容以外のものをリセットする
    pub fn clear_state(&mut self, engine: *mut IBusEngine) {
        if self.current_clause != 0 {
            self.current_clause = 0;
            self.on_current_clause_change(engine);
        }
        self.node_selected.clear();
        self.force_selected_clause.clear();
    }

    pub(crate) fn append_raw_input(&mut self, engine: *mut IBusEngine, ch: char) {
        self.raw_input.push(ch);
        self.on_raw_input_change(engine);
    }

    /// バックスペースで一文字削除した場合などに呼ばれる。
    pub(crate) fn set_raw_input(&mut self, engine: *mut IBusEngine, raw_input: String) {
        if self.raw_input != raw_input {
            self.raw_input = raw_input;
            self.on_raw_input_change(engine);
        }

        if !self.clauses.is_empty() {
            self.clauses.clear();
            self.on_clauses_change(engine);
        }

        self.clear_state(engine);
    }

    pub fn set_auxiliary_text(&mut self, engine: *mut IBusEngine, auxiliary_text: &str) {
        if self.auxiliary_text != auxiliary_text {
            self.auxiliary_text = auxiliary_text.to_string();
            self.on_auxiliary_text_change(engine);
        }
    }

    pub fn set_clauses(&mut self, engine: *mut IBusEngine, clause: Vec<Vec<Candidate>>) {
        if self.clauses != clause {
            self.clauses = clause;
            self.node_selected.clear();
            self.on_clauses_change(engine);
        }
    }

    /// 変換しているときに backspace を入力した場合。
    /// 変換候補をクリアして、Conversion から Composition 状態に戻る。
    pub fn clear_clauses(&mut self, engine: *mut IBusEngine) {
        if !self.clauses.is_empty() {
            self.clauses.clear();
            self.on_clauses_change(engine);
        }
        self.clear_state(engine);
    }

    /**
     * 現在の候補選択状態から、 lookup table を構築する。
     */
    fn render_lookup_table(&mut self) {
        info!("render_lookup_table");
        // 一旦、ルックアップテーブルをクリアする
        self.lookup_table.clear();

        // 現在の未変換情報を元に、候補を算出していく。
        if !self.clauses.is_empty() {
            // lookup table に候補を詰め込んでいく。
            for node in &self.clauses[self.current_clause] {
                let candidate = &node.surface_with_dynamic();
                self.lookup_table.append_candidate(candidate.to_ibus_text());
            }
        }
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
    pub fn select_right_clause(&mut self, engine: *mut IBusEngine) {
        if self.current_clause == self.clauses.len() - 1 {
            // 既に一番右だった場合、一番左にいく。
            if self.current_clause != 0 {
                self.current_clause = 0;
                self.on_current_clause_change(engine);
            }
        } else {
            self.current_clause += 1;
            self.on_current_clause_change(engine);
        }
    }

    /// 一個左の文節を選択する
    pub fn select_left_clause(&mut self, engine: *mut IBusEngine) {
        if self.current_clause == 0 {
            // 既に一番左だった場合、一番右にいく
            self.current_clause = self.clauses.len() - 1;
            self.on_current_clause_change(engine);
        } else {
            self.current_clause -= 1;
            self.on_current_clause_change(engine);
        }
    }

    pub fn adjust_current_clause(&mut self, engine: *mut IBusEngine) {
        // [a][bc]
        //    ^^^^
        // 上記の様にフォーカスが当たっている時に extend_clause_left した場合
        // 文節の数がもとより減ることがある。その場合は index error になってしまうので、
        // current_clause を動かす。
        if self.current_clause >= self.clauses.len() {
            self.current_clause = self.clauses.len() - 1;
            self.on_current_clause_change(engine);
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

    pub fn on_clauses_change(&mut self, engine: *mut IBusEngine) {
        self.render_preedit(engine);
        self.render_lookup_table();
    }

    pub fn on_raw_input_change(&self, _engine: *mut IBusEngine) {}

    pub fn on_current_clause_change(&mut self, engine: *mut IBusEngine) {
        self.render_preedit(engine);
        self.render_lookup_table();

        // -- auxiliary text(ポップアップしてるやつのほう)
        if !self.clauses.is_empty() {
            let current_yomi = self.clauses[self.current_clause][0].yomi.clone();
            self.set_auxiliary_text(engine, &current_yomi);
        }

        // 候補があれば、選択肢を表示させる。
        let visible = self.lookup_table.get_number_of_candidates() > 0;
        self.set_lookup_table_visible(engine, visible);
    }

    fn on_auxiliary_text_change(&self, engine: *mut IBusEngine) {
        self.render_auxiliary_text(engine);
    }

    pub fn render_preedit(&self, engine: *mut IBusEngine) {
        if self.clauses.is_empty() {
            unsafe { ibus_engine_hide_preedit_text(engine) }
            return;
        }

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

    pub(crate) fn get_key_state(&self) -> KeyState {
        // キー入力状態を返す。
        if self.raw_input.is_empty() {
            // 未入力状態。
            KeyState::PreComposition
        } else if self.in_conversion() {
            // 変換している状態。lookup table が表示されている状態
            KeyState::Conversion
        } else {
            // preedit になにか入っていて、まだ変換を実施していない状態
            KeyState::Composition
        }
    }

    fn render_auxiliary_text(&self, engine: *mut IBusEngine) {
        unsafe {
            if self.auxiliary_text.is_empty() {
                ibus_engine_hide_auxiliary_text(engine);
            } else {
                let auxiliary_text = self.auxiliary_text.to_ibus_text();
                ibus_text_set_attributes(auxiliary_text, ibus_attr_list_new());
                ibus_engine_update_auxiliary_text(
                    engine,
                    auxiliary_text,
                    to_gboolean(!self.raw_input.is_empty()),
                );
            }
        }
    }

    pub fn set_lookup_table_visible(&mut self, engine: *mut IBusEngine, visible: bool) {
        unsafe {
            ibus_engine_update_lookup_table(
                engine,
                &mut self.lookup_table as *mut IBusLookupTable,
                to_gboolean(visible),
            );
        }
    }
}
