use std::collections::HashMap;

use anyhow::Result;
use kelp::{h2z, hira2kata, z2h, ConvOption};
use log::{debug, error, info, trace, warn};

use akaza_conf::conf::open_configuration_window;
use ibus_sys::core::{
    IBusModifierType_IBUS_CONTROL_MASK, IBusModifierType_IBUS_HYPER_MASK,
    IBusModifierType_IBUS_META_MASK, IBusModifierType_IBUS_MOD1_MASK,
    IBusModifierType_IBUS_MOD2_MASK, IBusModifierType_IBUS_MOD3_MASK,
    IBusModifierType_IBUS_MOD4_MASK, IBusModifierType_IBUS_MOD5_MASK,
    IBusModifierType_IBUS_RELEASE_MASK, IBusModifierType_IBUS_SHIFT_MASK,
};
use ibus_sys::engine::ibus_engine_commit_text;
use ibus_sys::engine::IBusEngine;
use ibus_sys::glib::guint;
use ibus_sys::property::IBusPropState_PROP_STATE_CHECKED;
use ibus_sys::text::StringExt;
use libakaza::config::Config;
use libakaza::engine::base::HenkanEngine;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngine;
use libakaza::graph::candidate::Candidate;
use libakaza::kana_kanji::marisa_kana_kanji_dict::MarisaKanaKanjiDict;
use libakaza::lm::system_bigram::MarisaSystemBigramLM;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;
use libakaza::romkan::RomKanConverter;

use crate::commands::{ibus_akaza_commands_map, IbusAkazaCommand};
use crate::current_state::CurrentState;
use crate::input_mode::get_input_mode_from_prop_name;
use crate::input_mode::InputMode;
use crate::input_mode::INPUT_MODE_HIRAGANA;
use crate::keymap::KeyMap;
use crate::ui::prop_controller::PropController;

#[repr(C)]
pub struct AkazaContext {
    // ==== 設定 ====
    keymap: KeyMap,
    command_map: HashMap<&'static str, IbusAkazaCommand>,

    // ==== 現在の入力状態を保持 ====
    current_state: CurrentState,

    // ==== UI 関連 ====
    prop_controller: PropController,
}

impl AkazaContext {
    pub(crate) fn new(
        engine: BigramWordViterbiEngine<
            MarisaSystemUnigramLM,
            MarisaSystemBigramLM,
            MarisaKanaKanjiDict,
        >,
        config: Config,
    ) -> Result<Self> {
        let input_mode = INPUT_MODE_HIRAGANA;
        let romkan = RomKanConverter::new(config.romkan.as_str())?;

        Ok(AkazaContext {
            current_state: CurrentState::new(input_mode, config.live_conversion, romkan, engine),
            command_map: ibus_akaza_commands_map(),
            keymap: KeyMap::new(config.keymap)?,
            prop_controller: PropController::new(input_mode)?,
        })
    }

    /// Set props
    pub(crate) fn do_property_activate(
        &mut self,
        engine: *mut IBusEngine,
        prop_name: String,
        prop_state: guint,
    ) {
        debug!("do_property_activate: {}, {}", prop_name, prop_state);
        if prop_name == "PrefPane" {
            match open_configuration_window() {
                Ok(_) => {}
                Err(e) => info!("Err: {}", e),
            }
        } else if prop_state == IBusPropState_PROP_STATE_CHECKED
            && prop_name.starts_with("InputMode.")
        {
            self.input_mode_activate(engine, prop_name, prop_state);
        }
    }

    pub fn input_mode_activate(
        &mut self,
        engine: *mut IBusEngine,
        prop_name: String,
        _prop_state: guint,
    ) {
        if let Ok(input_mode) = get_input_mode_from_prop_name(prop_name.as_str()) {
            self.set_input_mode(engine, &input_mode);
        } else {
            warn!("Unknown prop_name: {}", prop_name);
        }
    }
}

impl AkazaContext {
    pub(crate) fn process_num_key(&mut self, nn: i32, engine: *mut IBusEngine) -> bool {
        let idx = if nn == 0 { 9 } else { nn - 1 };

        if self.current_state.lookup_table_visible {
            self.set_lookup_table_cursor_pos_in_current_page(engine, idx)
        } else {
            info!("ignore process_num_key. lookup table is not enabled.");
            false
        }
    }

    /// Sets the cursor in the lookup table to index in the current page
    /// Returns True if successful, False if not.
    fn set_lookup_table_cursor_pos_in_current_page(
        &mut self,
        engine: *mut IBusEngine,
        idx: i32,
    ) -> bool {
        trace!("set_lookup_table_cursor_pos_in_current_page: {}", idx);

        let page_size = self.current_state.lookup_table.get_page_size();
        if idx > (page_size as i32) {
            info!("Index too big: {} > {}", idx, page_size);
            return false;
        }

        let page = self.current_state.lookup_table.get_cursor_pos() / page_size;
        // let pos_in_page = self.lookup_table.get_cursor_pos() % page_size;

        let new_pos = page * page_size + (idx as u32);

        if new_pos >= self.current_state.lookup_table.get_number_of_candidates() {
            info!(
                "new_pos too big: {} > {}",
                new_pos,
                self.current_state.lookup_table.get_number_of_candidates()
            );
            return false;
        }
        self.current_state.lookup_table.set_cursor_pos(new_pos);
        let cursor_pos = self.current_state.lookup_table.get_cursor_pos() as usize;
        self.current_state.select_candidate(engine, cursor_pos);

        true
    }
}

impl AkazaContext {
    pub fn process_key_event(
        &mut self,
        engine: *mut IBusEngine,
        keyval: guint,
        keycode: guint,
        modifiers: guint,
    ) -> bool {
        trace!(
            "process_key_event: keyval={}, keycode={}, modifiers={}",
            keyval,
            keycode,
            modifiers
        );

        // ignore key release event
        if modifiers & IBusModifierType_IBUS_RELEASE_MASK != 0 {
            return false;
        }
        let key_state = self.current_state.get_key_state();

        trace!("KeyState={:?}", key_state);
        if let Some(callback) = self
            .keymap
            .get(
                &key_state,
                keyval,
                modifiers
                    & (IBusModifierType_IBUS_CONTROL_MASK
                        | IBusModifierType_IBUS_SHIFT_MASK
                        | IBusModifierType_IBUS_META_MASK
                        | IBusModifierType_IBUS_HYPER_MASK
                        | IBusModifierType_IBUS_MOD1_MASK
                        | IBusModifierType_IBUS_MOD2_MASK
                        | IBusModifierType_IBUS_MOD3_MASK
                        | IBusModifierType_IBUS_MOD4_MASK
                        | IBusModifierType_IBUS_MOD5_MASK),
            )
            .cloned()
        {
            if self.run_callback_by_name(engine, callback.as_str()) {
                return true;
            }
        }

        match self.current_state.input_mode.prop_name {
            "InputMode.Hiragana" | "InputMode.Katakana" | "InputMode.HalfWidthKatakana" => {
                if modifiers
                    & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK)
                    != 0
                {
                    return false;
                }

                if ('!' as u32) <= keyval && keyval <= ('~' as u32) {
                    trace!(
                        "Insert new character to preedit: '{}'",
                        self.current_state.get_raw_input()
                    );

                    if !self.current_state.live_conversion
                        && self.current_state.lookup_table.get_number_of_candidates() > 0
                    {
                        // 変換の途中に別の文字が入力された。
                        // よって、現在の preedit 文字列は確定させる。
                        self.commit_candidate(engine);
                    }

                    // 文字列を追加する。
                    let ch = char::from_u32(keyval).unwrap();
                    self.current_state.append_raw_input(engine, ch);

                    return true;
                }
            }
            "InputMode.Alphanumeric" => return false,
            "InputMode.FullWidthAlnum" => {
                if ('!' as u32) <= keyval
                    && keyval <= ('~' as u32)
                    && (modifiers
                        & (IBusModifierType_IBUS_CONTROL_MASK | IBusModifierType_IBUS_MOD1_MASK))
                        == 0
                {
                    let option = ConvOption {
                        ascii: true,
                        digit: true,
                        ..Default::default()
                    };
                    let text = h2z(char::from_u32(keyval).unwrap().to_string().as_str(), option);
                    unsafe { ibus_engine_commit_text(engine, text.to_ibus_text()) };
                    return true;
                }
            }
            _ => {
                warn!("Unknown prop: {}", self.current_state.input_mode.prop_name);
                return false;
            }
        }

        false // not proceeded
    }

    pub(crate) fn erase_character_before_cursor(&mut self, engine: *mut IBusEngine) {
        if !self.current_state.live_conversion && !self.current_state.clauses.is_empty() {
            // ライブ変換ではない時で変換フェーズな時に一文字消した場合は、変換状態から変換前の状態に戻す。

            // 変換候補をクリアする
            self.current_state.clear_clauses(engine);
            return;
        }

        // サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
        self.current_state.set_raw_input(
            engine,
            self.current_state
                .romkan
                .remove_last_char(self.current_state.get_raw_input()),
        )
    }
}

impl Drop for AkazaContext {
    fn drop(&mut self) {
        warn!("Dropping AkazaContext");
    }
}

impl AkazaContext {
    /**
     * 入力モードの変更
     */
    pub(crate) fn set_input_mode(&mut self, engine: *mut IBusEngine, input_mode: &InputMode) {
        info!("Changing input mode to : {:?}", input_mode);

        // 変換候補をいったんコミットする。
        self.commit_candidate(engine);

        self.prop_controller.set_input_mode(input_mode, engine);

        // 実際に input_mode を設定する
        self.current_state.set_input_mode(engine, input_mode);
    }

    pub(crate) fn run_callback_by_name(
        &mut self,
        engine: *mut IBusEngine,
        function_name: &str,
    ) -> bool {
        if let Some(function) = self.command_map.get(function_name) {
            info!("Calling function '{}'", function_name);
            function(self, engine)
        } else {
            error!("Unknown function '{}'", function_name);
            false
        }
    }

    pub fn commit_string(&mut self, engine: *mut IBusEngine, text: &str) {
        if !self.current_state.clauses.is_empty() {
            // 変換モードのときのみ学習を実施する
            self.current_state
                .engine
                .learn(self.current_state.get_first_candidates().as_slice());
        }

        unsafe {
            ibus_engine_commit_text(engine, text.to_ibus_text());
        }

        self.current_state.clear_raw_input(engine);
        self.current_state.update_lookup_table(engine, false);

        self.current_state.set_auxiliary_text(engine, "");
    }

    pub fn commit_candidate(&mut self, engine: *mut IBusEngine) {
        self.commit_string(engine, self.current_state.build_string().as_str());
    }

    // space key を押して、最初に変換に入る時の処理。
    pub(crate) fn update_candidates(&mut self, engine: *mut IBusEngine) -> bool {
        if self.current_state.get_raw_input().is_empty() {
            return false;
        }

        self.current_state.henkan(engine).unwrap();
        if self.current_state.clauses.is_empty() {
            // たぶん到達しないはず
            return true;
        }

        // -- auxiliary text(ポップアップしてるやつのほう)
        let current_yomi = self.current_state.clauses[self.current_state.current_clause][0]
            .yomi
            .clone();
        self.current_state.set_auxiliary_text(engine, &current_yomi);

        // 明示的に変換しているので、lookup table を表示する。
        self.current_state.update_lookup_table(engine, true);

        true
    }

    /// 前の変換候補を選択する。
    pub(crate) fn cursor_up(&mut self, engine: *mut IBusEngine) {
        if self.current_state.lookup_table.cursor_up() {
            let cursor_pos = self.current_state.lookup_table.get_cursor_pos() as usize;
            self.current_state.select_candidate(engine, cursor_pos);

            // lookup table の表示を更新する
            self.current_state.update_lookup_table(engine, true);
        }
    }

    /// 次の変換候補を選択する。
    pub fn cursor_down(&mut self, engine: *mut IBusEngine) {
        if self.current_state.lookup_table.cursor_down() {
            let cursor_pos = self.current_state.lookup_table.get_cursor_pos() as usize;
            self.current_state.select_candidate(engine, cursor_pos);
            // lookup table の表示を更新する
            self.current_state.update_lookup_table(engine, true);
        }
    }

    pub fn page_up(&mut self, engine: *mut IBusEngine) -> bool {
        if self.current_state.lookup_table.page_up() {
            let cursor_pos = self.current_state.lookup_table.get_cursor_pos() as usize;
            self.current_state.select_candidate(engine, cursor_pos);
            // lookup table の表示を更新する
            self.current_state.update_lookup_table(engine, true);
            true
        } else {
            false
        }
    }

    pub fn page_down(&mut self, engine: *mut IBusEngine) -> bool {
        if self.current_state.lookup_table.page_up() {
            let cursor_pos = self.current_state.lookup_table.get_cursor_pos() as usize;
            self.current_state.select_candidate(engine, cursor_pos);
            // lookup table の表示を更新する
            self.current_state.update_lookup_table(engine, true);
            true
        } else {
            false
        }
    }

    /// 選択する分節を右にずらす。
    pub(crate) fn cursor_right(&mut self, engine: *mut IBusEngine) {
        // 分節がない場合は、何もしない。
        if self.current_state.clauses.is_empty() {
            return;
        }

        self.current_state.select_right_clause(engine);
    }

    /// 選択する分節を左にずらす。
    pub(crate) fn cursor_left(&mut self, engine: *mut IBusEngine) {
        // 分節がなければ何もしない
        if self.current_state.clauses.is_empty() {
            return;
        }

        self.current_state.select_left_clause(engine);
    }

    /// 文節の選択範囲を右方向に広げる
    pub fn extend_clause_right(&mut self, engine: *mut IBusEngine) -> Result<()> {
        self.current_state.extend_right(engine);

        // -- auxiliary text(ポップアップしてるやつのほう)
        let current_yomi = self.current_state.clauses[self.current_state.current_clause][0]
            .yomi
            .clone();
        self.current_state.set_auxiliary_text(engine, &current_yomi);

        Ok(())
    }

    /// 文節の選択範囲を左方向に広げる
    pub fn extend_clause_left(&mut self, engine: *mut IBusEngine) -> Result<()> {
        self.current_state.extend_left(engine);

        // -- auxiliary text(ポップアップしてるやつのほう)
        let current_yomi = self.current_state.clauses[self.current_state.current_clause][0]
            .yomi
            .clone();
        self.current_state.set_auxiliary_text(engine, &current_yomi);

        Ok(())
    }

    pub fn do_candidate_clicked(
        &mut self,
        engine: *mut IBusEngine,
        index: guint,
        _button: guint,
        _state: guint,
    ) {
        info!("do_candidate_clicked");
        if self.set_lookup_table_cursor_pos_in_current_page(engine, index as i32) {
            self.commit_candidate(engine)
        }
    }

    pub fn do_focus_in(&mut self, engine: *mut IBusEngine) {
        trace!("do_focus_in");
        self.prop_controller.do_focus_in(engine);
    }

    /// convert selected word/characters to full-width hiragana (standard hiragana): ホワイト → ほわいと
    pub fn convert_to_full_hiragana(&mut self, engine: *mut IBusEngine) -> Result<()> {
        info!("Convert to full hiragana");
        let hira = self
            .current_state
            .romkan
            .to_hiragana(self.current_state.get_raw_input());
        self.convert_to_single(engine, hira.as_str(), hira.as_str())
    }

    /// convert to full-width katakana (standard katakana): ほわいと → ホワイト
    pub fn convert_to_full_katakana(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self
            .current_state
            .romkan
            .to_hiragana(self.current_state.get_raw_input());
        let kata = hira2kata(hira.as_str(), ConvOption::default());
        self.convert_to_single(engine, hira.as_str(), kata.as_str())
    }

    /// convert to half-width katakana (standard katakana): ほわいと → ﾎﾜｲﾄ
    pub fn convert_to_half_katakana(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self
            .current_state
            .romkan
            .to_hiragana(self.current_state.get_raw_input());
        let kata = z2h(
            hira2kata(hira.as_str(), ConvOption::default()).as_str(),
            ConvOption::default(),
        );
        self.convert_to_single(engine, hira.as_str(), kata.as_str())
    }

    /// convert to full-width romaji, all-capitals, proper noun capitalization (latin script inside
    /// Japanese text): ホワイト → ｈｏｗａｉｔｏ → ＨＯＷＡＩＴＯ → Ｈｏｗａｉｔｏ
    pub fn convert_to_full_romaji(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self
            .current_state
            .romkan
            .to_hiragana(self.current_state.get_raw_input());
        let romaji = h2z(
            self.current_state.get_raw_input(),
            ConvOption {
                kana: true,
                digit: true,
                ascii: true,
                ..Default::default()
            },
        );
        self.convert_to_single(engine, hira.as_str(), romaji.as_str())
    }

    /// convert to half-width romaji, all-capitals, proper noun capitalization (latin script like
    /// standard English): ホワイト → howaito → HOWAITO → Howaito
    pub fn convert_to_half_romaji(&mut self, engine: *mut IBusEngine) -> Result<()> {
        let hira = self
            .current_state
            .romkan
            .to_hiragana(self.current_state.get_raw_input());
        let romaji = z2h(
            self.current_state.get_raw_input(),
            ConvOption {
                kana: true,
                digit: true,
                ascii: true,
                ..Default::default()
            },
        );
        self.convert_to_single(engine, hira.as_str(), romaji.as_str())
    }

    /// 特定の1文節の文章を候補として表示する。
    /// F6 などを押した時用。
    fn convert_to_single(
        &mut self,
        engine: *mut IBusEngine,
        yomi: &str,
        surface: &str,
    ) -> Result<()> {
        // 候補を設定
        let candidate = Candidate::new(yomi, surface, 0_f32);
        self.current_state.clear_force_selected_clause(engine);
        self.current_state
            .set_clauses(engine, vec![Vec::from([candidate.clone()])]);

        // ルックアップテーブルに候補を設定
        self.current_state
            .set_auxiliary_text(engine, &candidate.yomi);

        // lookup table を表示させる
        self.current_state.update_lookup_table(engine, true);

        Ok(())
    }

    pub(crate) fn commit_preedit(&mut self, engine: *mut IBusEngine) {
        let (_, surface) = self.current_state.make_preedit_word_for_precomposition();
        self.commit_string(engine, surface.as_str());
    }

    pub fn escape(&mut self, engine: *mut IBusEngine) {
        trace!("escape");

        if self.current_state.live_conversion {
            self.current_state.clear_raw_input(engine);
        } else {
            // 変換候補の分節をクリアする。
            self.current_state.clear_clauses(engine);
        }
    }
}
