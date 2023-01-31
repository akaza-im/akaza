use std::collections::HashMap;
use std::ops::Range;

use kelp::{hira2kata, z2h, ConvOption};
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
use libakaza::consonant::ConsonantSuffixExtractor;
use libakaza::engine::base::HenkanEngine;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngine;
use libakaza::extend_clause::{extend_left, extend_right};
use libakaza::graph::candidate::Candidate;
use libakaza::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;
use libakaza::keymap::KeyState;
use libakaza::lm::system_bigram::MarisaSystemBigramLM;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;
use libakaza::romkan::RomKanConverter;

use crate::input_mode::{InputMode, INPUT_MODE_HALFWIDTH_KATAKANA, INPUT_MODE_KATAKANA};

#[derive(Debug)]
pub struct CurrentState {
    pub(crate) input_mode: InputMode,
    raw_input: String,
    preedit: String,
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
    pub romkan: RomKanConverter,
    pub(crate) engine:
        BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM, MarisaKanaKanjiDict>,
    consonant_suffix_extractor: ConsonantSuffixExtractor,
}

impl CurrentState {
    pub fn new(
        input_mode: InputMode,
        live_conversion: bool,
        romkan: RomKanConverter,
        engine: BigramWordViterbiEngine<
            MarisaSystemUnigramLM,
            MarisaSystemBigramLM,
            MarisaKanaKanjiDict,
        >,
    ) -> Self {
        CurrentState {
            input_mode,
            raw_input: String::new(),
            preedit: String::new(),
            auxiliary_text: String::new(),
            clauses: vec![],
            current_clause: 0,
            node_selected: HashMap::new(),
            force_selected_clause: Vec::new(),
            live_conversion,
            lookup_table_visible: false,
            lookup_table: IBusLookupTable::new(10, 0, 1, 1),
            romkan,
            engine,
            consonant_suffix_extractor: ConsonantSuffixExtractor::default(),
        }
    }

    pub(crate) fn set_input_mode(&mut self, engine: *mut IBusEngine, input_mode: &InputMode) {
        self.clear_raw_input(engine);
        self.clear_clauses(engine);
        self.input_mode = *input_mode;
    }

    pub fn select_candidate(&mut self, engine: *mut IBusEngine, candidate_pos: usize) {
        self.node_selected
            .insert(self.current_clause, candidate_pos);

        self.on_node_selected_change(engine);
    }

    pub fn clear_raw_input(&mut self, engine: *mut IBusEngine) {
        if !self.raw_input.is_empty() {
            self.raw_input.clear();
            self.on_raw_input_change(engine);
        }
    }

    pub fn get_raw_input(&self) -> &str {
        &self.raw_input
    }

    pub fn clear_force_selected_clause(&mut self, engine: *mut IBusEngine) {
        if !self.force_selected_clause.is_empty() {
            self.force_selected_clause.clear();
            self.on_force_selected_clause_change(engine);
        }
    }

    pub fn clear_current_clause(&mut self, engine: *mut IBusEngine) {
        if self.current_clause != 0 {
            self.current_clause = 0;
            self.on_current_clause_change(engine);
        }
    }

    pub(crate) fn append_raw_input(&mut self, engine: *mut IBusEngine, ch: char) {
        self.raw_input.push(ch);
        self.on_raw_input_change(engine);
    }

    /// バックスペースで一文字削除した場合などに呼ばれる。
    pub(crate) fn set_raw_input(&mut self, engine: *mut IBusEngine, raw_input: String) {
        if self.raw_input != raw_input {
            info!("set_raw_input: {:?}", raw_input);
            self.raw_input = raw_input;
            self.on_raw_input_change(engine);
        }
    }

    pub(crate) fn henkan(&mut self, engine: *mut IBusEngine) -> anyhow::Result<()> {
        if self.get_raw_input().is_empty() {
            self.set_clauses(engine, vec![]);
        } else {
            let yomi = self.get_raw_input().to_string();

            // 先頭が大文字なケースと、URL っぽい文字列のときは変換処理を実施しない。
            let clauses = if (!yomi.is_empty()
                && yomi.chars().next().unwrap().is_ascii_uppercase()
                && self.force_selected_clause.is_empty())
                || yomi.starts_with("https://")
                || yomi.starts_with("http://")
            {
                vec![Vec::from([Candidate::new(
                    yomi.as_str(),
                    yomi.as_str(),
                    0_f32,
                )])]
            } else {
                self.engine.convert(
                    self.romkan.to_hiragana(&yomi).as_str(),
                    Some(&self.force_selected_clause),
                )?
            };

            self.set_clauses(engine, clauses);

            self.adjust_current_clause(engine);
        }
        Ok(())
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
            self.clear_node_selected(engine);
            self.clear_current_clause(engine);
            self.on_clauses_change(engine);
        }
    }

    pub fn clear_node_selected(&mut self, engine: *mut IBusEngine) {
        if !self.node_selected.is_empty() {
            self.node_selected.clear();
            self.on_node_selected_change(engine);
        }
    }

    /// 変換しているときに backspace を入力した場合。
    /// 変換候補をクリアして、Conversion から Composition 状態に戻る。
    pub fn clear_clauses(&mut self, engine: *mut IBusEngine) {
        if !self.clauses.is_empty() {
            self.clauses.clear();
            self.on_clauses_change(engine);

            // lookup table を隠す
            self.update_lookup_table(engine, false);
        }
        self.clear_current_clause(engine);
        self.clear_node_selected(engine);
        self.clear_force_selected_clause(engine);
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

    pub fn extend_right(&mut self, engine: *mut IBusEngine) {
        self.force_selected_clause = extend_right(&self.clauses, self.current_clause);
        self.on_force_selected_clause_change(engine);
    }

    pub fn extend_left(&mut self, engine: *mut IBusEngine) {
        self.force_selected_clause = extend_left(&self.clauses, self.current_clause);
        self.on_force_selected_clause_change(engine);
    }

    pub fn on_force_selected_clause_change(&mut self, engine: *mut IBusEngine) {
        self.henkan(engine).unwrap();
    }

    pub fn on_clauses_change(&mut self, engine: *mut IBusEngine) {
        self.update_preedit(engine);
        self.update_auxiliary_text(engine);
        self.render_lookup_table();
    }

    pub fn on_raw_input_change(&mut self, engine: *mut IBusEngine) {
        if self.live_conversion {
            self.henkan(engine).unwrap();
        } else if !self.clauses.is_empty() {
            self.clauses.clear();
            self.on_clauses_change(engine);
        }

        self.clear_current_clause(engine);
        self.clear_node_selected(engine);
        self.clear_force_selected_clause(engine);

        self.update_preedit(engine);
    }

    pub fn on_current_clause_change(&mut self, engine: *mut IBusEngine) {
        self.update_preedit(engine);
        self.render_lookup_table();

        self.update_auxiliary_text(engine);

        // 候補があれば、選択肢を表示させる。
        let visible = self.lookup_table.get_number_of_candidates() > 0;
        self.update_lookup_table(engine, visible);
    }

    pub fn update_auxiliary_text(&mut self, engine: *mut IBusEngine) {
        // -- auxiliary text(ポップアップしてるやつのほう)
        if !self.clauses.is_empty() {
            let current_yomi = self.clauses[self.current_clause][0].yomi.clone();
            self.set_auxiliary_text(engine, &current_yomi);
        } else {
            self.set_auxiliary_text(engine, "");
        }
    }

    fn on_auxiliary_text_change(&self, engine: *mut IBusEngine) {
        self.render_auxiliary_text(engine);
    }

    pub fn update_preedit(&mut self, engine: *mut IBusEngine) {
        if self.live_conversion {
            if self.clauses.is_empty() {
                unsafe { ibus_engine_hide_preedit_text(engine) }
            } else {
                self.preedit = self.build_string();
                self.render_preedit(engine);
            }
        } else if self.clauses.is_empty() {
            // live conversion じゃなくて、変換中じゃないとき。
            let (_yomi, surface) = self.make_preedit_word_for_precomposition();
            self.preedit = surface;
            self.render_preedit(engine);
        } else {
            // live conversion じゃなくて、変換中のとき。
            self.preedit = self.build_string();
            self.render_preedit(engine);
        }
    }

    pub fn render_preedit(&self, engine: *mut IBusEngine) {
        unsafe {
            let preedit_attrs = ibus_attr_list_new();
            // 全部に下線をひく。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
                    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
                    0,
                    self.preedit.len() as guint,
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
                    bgstart + (self.preedit.len() as u32),
                ),
            );
            let preedit_text = self.preedit.to_ibus_text();
            ibus_text_set_attributes(preedit_text, preedit_attrs);
            ibus_engine_update_preedit_text(
                engine,
                preedit_text,
                self.preedit.len() as guint,
                to_gboolean(!self.preedit.is_empty()),
            );
        }
    }

    pub(crate) fn get_key_state(&self) -> KeyState {
        // キー入力状態を返す。
        if self.raw_input.is_empty() {
            // 未入力状態。
            KeyState::PreComposition
        } else if !self.clauses.is_empty() {
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

    /// lookup table の表示を更新する
    pub fn update_lookup_table(&mut self, engine: *mut IBusEngine, visible: bool) {
        self.lookup_table_visible = visible;
        unsafe {
            ibus_engine_update_lookup_table(
                engine,
                &mut self.lookup_table as *mut IBusLookupTable,
                to_gboolean(visible),
            );
        }
    }

    fn on_node_selected_change(&mut self, engine: *mut IBusEngine) {
        self.update_preedit(engine);
        self.update_auxiliary_text(engine);
    }

    /// (yomi, surface)
    pub fn make_preedit_word_for_precomposition(&self) -> (String, String) {
        let preedit = self.get_raw_input().to_string();
        // 先頭文字が大文字な場合は、そのまま返す。
        // "IME" などと入力された場合は、それをそのまま返すようにする。
        if !preedit.is_empty() && preedit.chars().next().unwrap().is_ascii_uppercase() {
            return (preedit.clone(), preedit);
        }

        // hogen と入力された場合、"ほげn" と表示する。
        // hogena となったら "ほげな"
        // hogenn となったら "ほげん" と表示する必要があるため。
        // 「ん」と一旦表示された後に「な」に変化したりすると気持ち悪く感じる。
        let (preedit, suffix) = if self.romkan.mapping_name == "default" {
            self.consonant_suffix_extractor.extract(preedit.as_str())
        } else {
            (preedit, "".to_string())
        };

        let yomi = self.romkan.to_hiragana(preedit.as_str());
        let surface = yomi.clone();
        if self.input_mode == INPUT_MODE_KATAKANA {
            (
                yomi.to_string() + suffix.as_str(),
                hira2kata(yomi.as_str(), ConvOption::default()) + suffix.as_str(),
            )
        } else if self.input_mode == INPUT_MODE_HALFWIDTH_KATAKANA {
            (
                yomi.to_string() + suffix.as_str(),
                z2h(
                    hira2kata(yomi.as_str(), ConvOption::default()).as_str(),
                    ConvOption::default(),
                ) + suffix.as_str(),
            )
        } else {
            (yomi + suffix.as_str(), surface + suffix.as_str())
        }
    }
}
