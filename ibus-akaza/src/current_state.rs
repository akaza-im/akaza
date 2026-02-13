use std::collections::HashMap;
use std::ops::Range;

use kelp::{hira2kata, z2h, ConvOption};
use log::{error, info};

use ibus_sys::attr_list::{ibus_attr_list_append, ibus_attr_list_new};
use ibus_sys::attribute::{
    ibus_attribute_new, IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
    IBusAttrType_IBUS_ATTR_TYPE_FOREGROUND, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
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
// 文節伸縮・選択の仕様は docs/clause-extension-behavior.md を参照。
use libakaza::graph::candidate::Candidate;
use libakaza::graph::graph_resolver::KBestPath;
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
    /// サジェストによる自動変換中かどうか（Space で明示的に変換した場合は false）
    pub(crate) suggest_active: bool,
    /// サジェスト中に Tab/Up/Down で候補を選択したかどうか
    /// true の場合、preedit に変換結果を表示し、Enter で確定する
    pub(crate) suggest_candidate_selected: bool,
    pub(crate) lookup_table_visible: bool,
    pub lookup_table: IBusLookupTable,
    pub romkan: RomKanConverter,
    pub(crate) engine:
        BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM, MarisaKanaKanjiDict>,
    consonant_suffix_extractor: ConsonantSuffixExtractor,
    /// k-best の分節パターン候補
    segmentation_alternatives: Vec<KBestPath>,
    /// 現在選択中の分節パターン (0 = 1-best)
    current_segmentation: usize,
}

fn next_clause_index(current: usize, len: usize, dir: i32) -> usize {
    if len <= 1 {
        return 0;
    }
    if dir >= 0 {
        if current + 1 >= len {
            0
        } else {
            current + 1
        }
    } else if current == 0 {
        len - 1
    } else {
        current - 1
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
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
            suggest_active: false,
            suggest_candidate_selected: false,
            lookup_table_visible: false,
            lookup_table: IBusLookupTable::new(10, 0, 1, 1),
            romkan,
            engine,
            consonant_suffix_extractor: ConsonantSuffixExtractor::default(),
            segmentation_alternatives: Vec::new(),
            current_segmentation: 0,
        }
    }

    /// サジェスト表示すべきかどうかを判定する。
    /// ひらがな2文字以上入力されている場合に true を返す。
    fn should_suggest(&self) -> bool {
        self.romkan.to_hiragana(&self.raw_input).chars().count() >= 2
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
        self.suggest_active = false;
        self.suggest_candidate_selected = false;
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
            self.segmentation_alternatives.clear();
            self.current_segmentation = 0;
            self.set_clauses(engine, vec![]);
        } else {
            let yomi = self.get_raw_input().to_string();

            // 先頭が大文字なケースと、URL っぽい文字列のときは変換処理を実施しない。
            if (!yomi.is_empty()
                && yomi.starts_with(|c: char| c.is_ascii_uppercase())
                && self.force_selected_clause.is_empty())
                || yomi.starts_with("https://")
                || yomi.starts_with("http://")
            {
                self.segmentation_alternatives.clear();
                self.current_segmentation = 0;
                let clauses = vec![Vec::from([Candidate::new(
                    yomi.as_str(),
                    yomi.as_str(),
                    0_f32,
                )])];
                self.set_clauses(engine, clauses);
            } else {
                let paths = self.engine.convert_k_best(
                    self.romkan.to_hiragana(&yomi).as_str(),
                    Some(&self.force_selected_clause),
                    5,
                )?;
                self.segmentation_alternatives = paths;
                self.current_segmentation = 0;
                let clauses = self
                    .segmentation_alternatives
                    .first()
                    .map(|p| p.segments.clone())
                    .unwrap_or_default();
                self.set_clauses(engine, clauses);
            };

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
            // When force_selected_clause is active (e.g. Shift+→/←),
            // keep the current clause selection to match typical IME behavior.
            if self.force_selected_clause.is_empty() {
                self.clear_current_clause(engine);
            }
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
        self.suggest_active = false;
        self.suggest_candidate_selected = false;
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

        if self.suggest_active {
            // サジェスト中は k-best の各分節パターンの変換結果全文を表示する
            let empty = HashMap::new();
            for alt in &self.segmentation_alternatives {
                let s = build_string_from_clauses(&alt.segments, &empty);
                self.lookup_table.append_candidate(s.to_ibus_text());
            }
        } else {
            // 現在の未変換情報を元に、候補を算出していく。
            if let Some(clause) = self.clauses.get(self.current_clause) {
                // lookup table に候補を詰め込んでいく。
                for node in clause {
                    let candidate = &node.surface_with_dynamic();
                    self.lookup_table.append_candidate(candidate.to_ibus_text());
                }
            }
        }
    }

    /// サジェストから Conversion に遷移する際に、文節内候補で lookup table を再構築する
    pub fn render_lookup_table_for_conversion(&mut self) {
        self.lookup_table.clear();
        if let Some(clause) = self.clauses.get(self.current_clause) {
            for node in clause {
                let candidate = &node.surface_with_dynamic();
                self.lookup_table.append_candidate(candidate.to_ibus_text());
            }
        }
    }

    pub fn get_first_candidates(&self) -> Vec<Candidate> {
        collect_first_candidates(&self.clauses, &self.node_selected)
    }

    /// 一個右の文節を選択する
    pub fn select_right_clause(&mut self, engine: *mut IBusEngine) {
        if self.clauses.is_empty() {
            return;
        }
        let next = next_clause_index(self.current_clause, self.clauses.len(), 1);
        if next != self.current_clause {
            self.current_clause = next;
            self.on_current_clause_change(engine);
        }
    }

    /// 一個左の文節を選択する
    pub fn select_left_clause(&mut self, engine: *mut IBusEngine) {
        if self.clauses.is_empty() {
            return;
        }
        let next = next_clause_index(self.current_clause, self.clauses.len(), -1);
        if next != self.current_clause {
            self.current_clause = next;
            self.on_current_clause_change(engine);
        }
    }

    pub fn adjust_current_clause(&mut self, engine: *mut IBusEngine) {
        // [a][bc]
        //    ^^^^
        // 上記の様にフォーカスが当たっている時に extend_clause_left した場合
        // 文節の数がもとより減ることがある。その場合は index error になってしまうので、
        // current_clause を動かす。
        if self.clauses.is_empty() {
            if self.current_clause != 0 {
                self.current_clause = 0;
                self.on_current_clause_change(engine);
            }
        } else if self.current_clause >= self.clauses.len() {
            self.current_clause = self.clauses.len() - 1;
            self.on_current_clause_change(engine);
        }
    }

    pub fn build_string(&self) -> String {
        build_string_from_clauses(&self.clauses, &self.node_selected)
    }

    pub fn extend_right(&mut self, engine: *mut IBusEngine) {
        self.current_segmentation = 0;
        self.force_selected_clause = extend_right(&self.clauses, self.current_clause);
        self.on_force_selected_clause_change(engine);
    }

    pub fn extend_left(&mut self, engine: *mut IBusEngine) {
        self.current_segmentation = 0;
        self.force_selected_clause = extend_left(&self.clauses, self.current_clause);
        self.on_force_selected_clause_change(engine);
    }

    /// サジェスト中に次の k-best パターンを選択する
    pub fn suggest_select_next(&mut self, engine: *mut IBusEngine) -> bool {
        if self.segmentation_alternatives.len() <= 1 {
            return false;
        }
        if self.lookup_table.cursor_down() {
            let idx = self.lookup_table.get_cursor_pos() as usize;
            self.current_segmentation = idx;
            self.clauses = self.segmentation_alternatives[idx].segments.clone();
            self.node_selected.clear();
            self.current_clause = 0;
            self.suggest_candidate_selected = true;
            self.update_lookup_table(engine, true);
            self.update_preedit(engine);
        }
        true
    }

    /// サジェスト中に前の k-best パターンを選択する
    pub fn suggest_select_prev(&mut self, engine: *mut IBusEngine) -> bool {
        if self.segmentation_alternatives.len() <= 1 {
            return false;
        }
        if self.lookup_table.cursor_up() {
            let idx = self.lookup_table.get_cursor_pos() as usize;
            self.current_segmentation = idx;
            self.clauses = self.segmentation_alternatives[idx].segments.clone();
            self.node_selected.clear();
            self.current_clause = 0;
            self.suggest_candidate_selected = true;
            self.update_lookup_table(engine, true);
            self.update_preedit(engine);
        }
        true
    }

    /// Tab キーで分節パターンを切り替える
    /// 切り替え候補がない場合は false を返す
    pub fn cycle_segmentation(&mut self, engine: *mut IBusEngine) -> bool {
        if self.segmentation_alternatives.len() <= 1 {
            return false;
        }
        self.current_segmentation =
            (self.current_segmentation + 1) % self.segmentation_alternatives.len();
        self.clauses = self.segmentation_alternatives[self.current_segmentation]
            .segments
            .clone();
        self.node_selected.clear();
        self.current_clause = 0;
        self.on_clauses_change(engine);
        // サジェスト中は lookup table のカーソルを現在の分節パターンに合わせる
        if self.suggest_active {
            self.suggest_candidate_selected = true;
            self.lookup_table
                .set_cursor_pos(self.current_segmentation as u32);
            self.update_lookup_table(engine, true);
            self.update_preedit(engine);
        }
        true
    }

    pub fn on_force_selected_clause_change(&mut self, engine: *mut IBusEngine) {
        if let Err(e) = self.henkan(engine) {
            error!("on_force_selected_clause_change: henkan failed: {}", e);
        }
    }

    pub fn on_clauses_change(&mut self, engine: *mut IBusEngine) {
        self.update_preedit(engine);
        self.update_auxiliary_text(engine);
        self.render_lookup_table();
    }

    pub fn on_raw_input_change(&mut self, engine: *mut IBusEngine) {
        // unicode character の境界じゃないところに force_selected が入った状態で hanken
        // すると落ちる。
        // なので、先にクリアする必要がある。
        self.clear_force_selected_clause(engine);

        if self.live_conversion {
            if let Err(e) = self.henkan(engine) {
                error!("on_raw_input_change: henkan failed: {}", e);
            }
        } else if self.should_suggest() {
            // サジェストモード: ひらがな2文字以上で候補を表示
            // henkan 内で on_clauses_change → render_preedit が呼ばれるため、
            // 先に suggest_active をセットしておく
            self.suggest_active = true;
            self.suggest_candidate_selected = false;
            if let Err(e) = self.henkan(engine) {
                error!("on_raw_input_change: suggest henkan failed: {}", e);
            }
        } else if !self.clauses.is_empty() {
            // まだ2文字未満: clauses をクリア
            self.suggest_active = false;
            self.suggest_candidate_selected = false;
            self.clauses.clear();
            self.on_clauses_change(engine);
        }

        self.clear_current_clause(engine);
        self.clear_node_selected(engine);

        self.update_preedit(engine);

        let visible = !self.live_conversion && self.lookup_table.get_number_of_candidates() > 0;
        self.update_lookup_table(engine, visible);
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
        if self.suggest_active {
            // サジェスト中は入力全体のひらがなを表示
            let yomi = self.romkan.to_hiragana(&self.raw_input);
            self.set_auxiliary_text(engine, &yomi);
        } else if let Some(clause) = self.clauses.get(self.current_clause) {
            if let Some(first) = clause.first() {
                self.set_auxiliary_text(engine, &first.yomi.clone());
            } else {
                self.set_auxiliary_text(engine, "");
            }
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
        } else if self.suggest_active && self.suggest_candidate_selected {
            // サジェスト中に Tab/Up/Down で候補を選択した → 変換結果を表示
            self.preedit = self.build_string();
            self.render_preedit(engine);
        } else if self.suggest_active {
            // サジェスト中だがまだ候補を選択していない → ひらがな表示のまま
            let (_yomi, surface) = self.make_preedit_word_for_precomposition();
            self.preedit = surface;
            self.render_preedit(engine);
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
            // IBus の属性位置とカーソル位置は文字数（Unicode コードポイント数）で指定する。
            // Rust の String::len() は UTF-8 バイト長を返すため、chars().count() を使う。
            let preedit_char_len = self.preedit.chars().count() as guint;
            // 全部に下線をひく。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
                    IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE,
                    0,
                    preedit_char_len,
                ),
            );
            // サジェスト中（候補未選択時）は preedit がひらがなで clauses が漢字のため、
            // clauses から bgstart を計算するとずれる。
            // この場合は bgstart=0 にして通常の Composition と同じスタイルにする。
            let bgstart: u32 = if self.suggest_active && !self.suggest_candidate_selected {
                0
            } else {
                self.clauses
                    .iter()
                    .filter_map(|c| c.first())
                    .map(|c| c.surface.chars().count() as u32)
                    .sum()
            };
            // 背景色を設定する（bgstart から preedit 末尾まで）。
            // end_index は preedit 全体の文字数を超えてはならない。
            // 以前は bgstart + preedit_char_len としていたため、プリエディット文字列の
            // 長さを超える属性範囲が VTE (libvte) に渡り、fudge_pango_colors() 内で
            // 整数アンダーフローによるスタックオーバーフローを引き起こし、
            // gnome-terminal が SIGSEGV でクラッシュしていた。
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_BACKGROUND,
                    0x00FFFFFF,
                    bgstart,
                    preedit_char_len,
                ),
            );
            ibus_attr_list_append(
                preedit_attrs,
                ibus_attribute_new(
                    IBusAttrType_IBUS_ATTR_TYPE_FOREGROUND,
                    0x00000000,
                    bgstart,
                    preedit_char_len,
                ),
            );
            let preedit_text = self.preedit.to_ibus_text();
            ibus_text_set_attributes(preedit_text, preedit_attrs);
            ibus_engine_update_preedit_text(
                engine,
                preedit_text,
                preedit_char_len,
                to_gboolean(!self.preedit.is_empty()),
            );
        }
    }

    pub(crate) fn get_key_state(&self) -> KeyState {
        // キー入力状態を返す。
        if self.raw_input.is_empty() {
            // 未入力状態。
            KeyState::PreComposition
        } else if self.suggest_active && self.suggest_candidate_selected {
            // サジェスト中に候補を選択した → Conversion として扱う
            // → Enter で commit_candidate が効く
            KeyState::Conversion
        } else if self.suggest_active {
            // サジェスト中だが候補未選択 → Composition を維持
            // → Space キーで update_candidates が効く
            KeyState::Composition
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
            if self.lookup_table_visible {
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
            } else {
                ibus_engine_hide_auxiliary_text(engine);
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
        if preedit.starts_with(|c: char| c.is_ascii_uppercase()) {
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

/// clauses と node_selected から、各文節の選択された候補を集める。
/// CurrentState に依存しないため、単体テストが可能。
fn collect_first_candidates(
    clauses: &[Vec<Candidate>],
    node_selected: &HashMap<usize, usize>,
) -> Vec<Candidate> {
    let mut targets: Vec<Candidate> = Vec::new();
    for (i, candidates) in clauses.iter().enumerate() {
        if candidates.is_empty() {
            error!(
                "[BUG] get_first_candidates: clause {} has no candidates, skipping.",
                i
            );
            continue;
        }
        let idx = node_selected.get(&i).unwrap_or(&0);
        let safe_idx = if *idx >= candidates.len() {
            error!(
                "[BUG] get_first_candidates: node_selected index out of bounds: clause={}, idx={}, candidates.len()={}. Using index 0 as fallback.",
                i, idx, candidates.len()
            );
            0
        } else {
            *idx
        };
        targets.push(candidates[safe_idx].clone());
    }
    targets
}

/// clauses と node_selected から、変換結果の文字列を構築する。
/// CurrentState に依存しないため、単体テストが可能。
fn build_string_from_clauses(
    clauses: &[Vec<Candidate>],
    node_selected: &HashMap<usize, usize>,
) -> String {
    let mut result = String::new();
    for (clauseid, nodes) in clauses.iter().enumerate() {
        let idex = if let Some(i) = node_selected.get(&clauseid) {
            *i
        } else {
            0
        };

        // インデックスが範囲外の場合、安全に0番目の候補にフォールバック
        let safe_idex = if idex >= nodes.len() {
            error!(
                "[BUG] node_selected index out of bounds: clauseid={}, idex={}, nodes.len()={}. Using index 0 as fallback.",
                clauseid, idex, nodes.len()
            );
            0
        } else {
            idex
        };

        result += &nodes[safe_idex].surface_with_dynamic();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(yomi: &str, surface: &str) -> Candidate {
        Candidate::new(yomi, surface, 0_f32)
    }

    // --- collect_first_candidates tests ---

    #[test]
    fn test_collect_first_candidates_normal() {
        let clauses = vec![
            vec![candidate("きょう", "今日"), candidate("きょう", "京")],
            vec![candidate("は", "は"), candidate("は", "葉")],
        ];
        let node_selected = HashMap::new();
        let result = collect_first_candidates(&clauses, &node_selected);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].surface, "今日");
        assert_eq!(result[1].surface, "は");
    }

    #[test]
    fn test_collect_first_candidates_with_selection() {
        let clauses = vec![
            vec![candidate("きょう", "今日"), candidate("きょう", "京")],
            vec![candidate("は", "は"), candidate("は", "葉")],
        ];
        let mut node_selected = HashMap::new();
        node_selected.insert(0, 1); // 2番目の候補を選択
        let result = collect_first_candidates(&clauses, &node_selected);
        assert_eq!(result[0].surface, "京");
        assert_eq!(result[1].surface, "は");
    }

    #[test]
    fn test_collect_first_candidates_index_out_of_bounds() {
        let clauses = vec![vec![candidate("きょう", "今日")]];
        let mut node_selected = HashMap::new();
        node_selected.insert(0, 99); // 範囲外
        let result = collect_first_candidates(&clauses, &node_selected);
        // panic せず、0番目にフォールバック
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].surface, "今日");
    }

    #[test]
    fn test_collect_first_candidates_empty_clause() {
        let clauses: Vec<Vec<Candidate>> = vec![
            vec![candidate("きょう", "今日")],
            vec![], // 空の文節
            vec![candidate("です", "です")],
        ];
        let node_selected = HashMap::new();
        let result = collect_first_candidates(&clauses, &node_selected);
        // 空の文節はスキップされる
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].surface, "今日");
        assert_eq!(result[1].surface, "です");
    }

    #[test]
    fn test_collect_first_candidates_empty_clauses() {
        let clauses: Vec<Vec<Candidate>> = vec![];
        let node_selected = HashMap::new();
        let result = collect_first_candidates(&clauses, &node_selected);
        assert!(result.is_empty());
    }

    // --- build_string_from_clauses tests ---

    #[test]
    fn test_build_string_normal() {
        let clauses = vec![
            vec![candidate("きょう", "今日"), candidate("きょう", "京")],
            vec![candidate("は", "は")],
        ];
        let node_selected = HashMap::new();
        let result = build_string_from_clauses(&clauses, &node_selected);
        assert_eq!(result, "今日は");
    }

    #[test]
    fn test_build_string_with_selection() {
        let clauses = vec![
            vec![candidate("きょう", "今日"), candidate("きょう", "京")],
            vec![candidate("は", "は"), candidate("は", "葉")],
        ];
        let mut node_selected = HashMap::new();
        node_selected.insert(0, 1);
        node_selected.insert(1, 1);
        let result = build_string_from_clauses(&clauses, &node_selected);
        assert_eq!(result, "京葉");
    }

    #[test]
    fn test_build_string_index_out_of_bounds() {
        let clauses = vec![
            vec![candidate("きょう", "今日")],
            vec![candidate("は", "は")],
        ];
        let mut node_selected = HashMap::new();
        node_selected.insert(0, 50); // 範囲外
        let result = build_string_from_clauses(&clauses, &node_selected);
        // panic せず、0番目にフォールバック
        assert_eq!(result, "今日は");
    }

    #[test]
    fn test_build_string_empty_clauses() {
        let clauses: Vec<Vec<Candidate>> = vec![];
        let node_selected = HashMap::new();
        let result = build_string_from_clauses(&clauses, &node_selected);
        assert_eq!(result, "");
    }

    #[test]
    fn test_next_clause_index_right_wraps() {
        assert_eq!(next_clause_index(0, 2, 1), 1);
        assert_eq!(next_clause_index(1, 2, 1), 0);
    }

    #[test]
    fn test_next_clause_index_left_wraps() {
        assert_eq!(next_clause_index(0, 2, -1), 1);
        assert_eq!(next_clause_index(1, 2, -1), 0);
    }

    #[test]
    fn test_next_clause_index_single() {
        assert_eq!(next_clause_index(0, 1, 1), 0);
        assert_eq!(next_clause_index(0, 1, -1), 0);
    }
}
