use std::collections::HashMap;
use std::process::Command;

use anyhow::Result;
use kelp::{h2z, hira2kata, z2h, ConvOption};
use log::{error, info, trace, warn};

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
use libakaza::keymap::Keymap;
use libakaza::lm::system_bigram::MarisaSystemBigramLM;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;
use libakaza::romkan::RomKanConverter;

use crate::commands::{ibus_akaza_commands_map, IbusAkazaCommand};
use crate::current_state::CurrentState;
use crate::input_mode::get_input_mode_from_prop_name;
use crate::input_mode::InputMode;
use crate::input_mode::INPUT_MODE_HIRAGANA;
use crate::keymap::IBusKeyMap;
use crate::ui::prop_controller::PropController;

#[repr(C)]
pub struct AkazaContext {
    // ==== 設定 ====
    keymap: IBusKeyMap,
    command_map: HashMap<&'static str, IbusAkazaCommand>,

    // ==== 現在の入力状態を保持 ====
    current_state: CurrentState,

    // ==== UI 関連 ====
    prop_controller: PropController,
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AkazaContext {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(
        engine: BigramWordViterbiEngine<
            MarisaSystemUnigramLM,
            MarisaSystemBigramLM,
            MarisaKanaKanjiDict,
        >,
        config: Config,
    ) -> Result<Self> {
        let input_mode = INPUT_MODE_HIRAGANA;
        let romkan = RomKanConverter::new(config.romkan.as_str())?;
        let keymap = Keymap::load(config.keymap.as_str())?;

        Ok(AkazaContext {
            current_state: CurrentState::new(input_mode, config.live_conversion, romkan, engine),
            command_map: ibus_akaza_commands_map(),
            keymap: IBusKeyMap::new(keymap)?,
            prop_controller: PropController::new(input_mode, config)?,
        })
    }

    /// Set props
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn do_property_activate(
        &mut self,
        engine: *mut IBusEngine,
        prop_name: String,
        prop_state: guint,
    ) {
        info!("do_property_activate: {}, {}", prop_name, prop_state);
        if prop_name == "PrefPane" {
            info!("PrefPane matched. Attempting to launch akaza-conf...");
            let which_result = Command::new("which").arg("akaza-conf").output();
            match &which_result {
                Ok(output) => {
                    info!(
                        "which akaza-conf: status={}, stdout={}, stderr={}",
                        output.status,
                        String::from_utf8_lossy(&output.stdout).trim(),
                        String::from_utf8_lossy(&output.stderr).trim()
                    );
                }
                Err(e) => {
                    warn!("which akaza-conf failed: {}", e);
                }
            }
            info!("Launching akaza-conf process...");
            match Command::new("akaza-conf").spawn() {
                Ok(child) => {
                    info!("akaza-conf process spawned: pid={}", child.id());
                }
                Err(e) => {
                    error!("akaza-conf launch failed: {}", e);
                }
            }
        } else if prop_state == IBusPropState_PROP_STATE_CHECKED
            && prop_name.starts_with("InputMode.")
        {
            self.input_mode_activate(engine, prop_name, prop_state);
        } else if prop_name.starts_with("UserDict.") {
            info!("Starting UserDict. operation");
            let Some(dict_path) = self.prop_controller.user_dict_path(&prop_name) else {
                warn!("Unknown user dict prop_name: {}", prop_name);
                return;
            };
            info!("Edit the {}", dict_path);

            // akaza-dict は別プロセスで起動し、辞書GUIの不具合が IME 本体のクラッシュに
            // 波及しないようにする（プロセス分離）。
            info!("Launching akaza-dict process...");
            match Command::new("akaza-dict").arg(dict_path).spawn() {
                Ok(child) => {
                    info!("akaza-dict process spawned: pid={}", child.id());
                }
                Err(e) => {
                    error!("akaza-dict launch failed: {}", e);
                }
            }
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

#[allow(clippy::not_unsafe_ptr_arg_deref)]
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

#[allow(clippy::not_unsafe_ptr_arg_deref)]
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

                    if self.current_state.lookup_table_visible
                        && !self.current_state.suggest_active
                        && !self.current_state.suggest_candidate_selected
                    {
                        // 変換の途中に別の文字が入力された。
                        // よって、現在の preedit 文字列は確定させる。
                        // （サジェスト中はまだ Composition なので確定しない）
                        self.commit_candidate(engine);
                    }

                    // 文字列を追加する。
                    let Some(ch) = keyval_to_char(keyval) else {
                        warn!("Invalid keyval: 0x{:X}", keyval);
                        return false;
                    };
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
                    let Some(ch) = keyval_to_char(keyval) else {
                        warn!("Invalid keyval: 0x{:X}", keyval);
                        return false;
                    };
                    let text = h2z(ch.to_string().as_str(), option);
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
        if !self.current_state.live_conversion
            && !self.current_state.suggest_active
            && !self.current_state.clauses.is_empty()
        {
            // ライブ変換でもサジェスト中でもない時で変換フェーズな時に一文字消した場合は、変換状態から変換前の状態に戻す。

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

#[allow(clippy::not_unsafe_ptr_arg_deref)]
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

        if self.current_state.suggest_active {
            // サジェスト中は既に clauses があるので、そのまま Conversion に遷移
            self.current_state.suggest_active = false;
            self.current_state.suggest_candidate_selected = false;
        } else if let Err(e) = self.current_state.henkan(engine) {
            error!("update_candidates: henkan failed: {}", e);
            return false;
        }
        if self.current_state.clauses.is_empty() {
            // たぶん到達しないはず
            return true;
        }

        // Conversion 用の lookup table を再構築（文節内の候補一覧）
        self.current_state.render_lookup_table_for_conversion();

        // -- auxiliary text(ポップアップしてるやつのほう)
        if let Some(clause) = self
            .current_state
            .clauses
            .get(self.current_state.current_clause)
        {
            if let Some(first) = clause.first() {
                let current_yomi = first.yomi.clone();
                self.current_state.set_auxiliary_text(engine, &current_yomi);
            }
        }

        // preedit を変換結果表示に更新
        self.current_state.update_preedit(engine);

        // 明示的に変換しているので、lookup table を表示する。
        self.current_state.update_lookup_table(engine, true);

        true
    }

    /// 前の変換候補を選択する。
    pub(crate) fn cursor_up(&mut self, engine: *mut IBusEngine) -> bool {
        if self.current_state.suggest_active {
            return self.suggest_select_prev(engine);
        }
        if self.current_state.lookup_table.cursor_up() {
            let cursor_pos = self.current_state.lookup_table.get_cursor_pos() as usize;
            self.current_state.select_candidate(engine, cursor_pos);

            // lookup table の表示を更新する
            self.current_state.update_lookup_table(engine, true);
        }
        true
    }

    /// 次の変換候補を選択する。
    pub fn cursor_down(&mut self, engine: *mut IBusEngine) -> bool {
        if self.current_state.suggest_active {
            return self.suggest_select_next(engine);
        }
        if self.current_state.lookup_table.cursor_down() {
            let cursor_pos = self.current_state.lookup_table.get_cursor_pos() as usize;
            self.current_state.select_candidate(engine, cursor_pos);
            // lookup table の表示を更新する
            self.current_state.update_lookup_table(engine, true);
        }
        true
    }

    /// サジェスト中に次の k-best パターンを選択する
    fn suggest_select_next(&mut self, engine: *mut IBusEngine) -> bool {
        self.current_state.suggest_select_next(engine)
    }

    /// サジェスト中に前の k-best パターンを選択する
    fn suggest_select_prev(&mut self, engine: *mut IBusEngine) -> bool {
        self.current_state.suggest_select_prev(engine)
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
        if self.current_state.lookup_table.page_down() {
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
        if let Some(clause) = self
            .current_state
            .clauses
            .get(self.current_state.current_clause)
        {
            if let Some(first) = clause.first() {
                let current_yomi = first.yomi.clone();
                self.current_state.set_auxiliary_text(engine, &current_yomi);
            }
        }

        Ok(())
    }

    /// 分節パターンを切り替える
    /// 切り替え候補がない場合は false を返す（キーをアプリに渡す）
    pub fn cycle_segmentation(&mut self, engine: *mut IBusEngine) -> bool {
        self.current_state.cycle_segmentation(engine)
    }

    /// 文節の選択範囲を左方向に広げる
    pub fn extend_clause_left(&mut self, engine: *mut IBusEngine) -> Result<()> {
        self.current_state.extend_left(engine);

        // -- auxiliary text(ポップアップしてるやつのほう)
        if let Some(clause) = self
            .current_state
            .clauses
            .get(self.current_state.current_clause)
        {
            if let Some(first) = clause.first() {
                let current_yomi = first.yomi.clone();
                self.current_state.set_auxiliary_text(engine, &current_yomi);
            }
        }

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
        } else if self.current_state.suggest_active {
            // サジェスト中は clauses をクリアして suggest_active をリセット
            self.current_state.clear_clauses(engine);
        } else {
            // 変換候補の分節をクリアする。
            self.current_state.clear_clauses(engine);
        }
    }
}

/// keyval を char に安全に変換する
///
/// 無効な Unicode コードポイントの場合は None を返す
fn keyval_to_char(keyval: guint) -> Option<char> {
    char::from_u32(keyval)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyval_to_char_valid() {
        // 通常のASCII文字
        assert_eq!(keyval_to_char('a' as u32), Some('a'));
        assert_eq!(keyval_to_char('Z' as u32), Some('Z'));
        assert_eq!(keyval_to_char('0' as u32), Some('0'));

        // 日本語文字
        assert_eq!(keyval_to_char('あ' as u32), Some('あ'));
        assert_eq!(keyval_to_char('漢' as u32), Some('漢'));
    }

    #[test]
    fn test_keyval_to_char_invalid() {
        // サロゲートペア範囲（無効）
        assert_eq!(keyval_to_char(0xD800), None);
        assert_eq!(keyval_to_char(0xDFFF), None);

        // Unicodeの範囲外
        assert_eq!(keyval_to_char(0x110000), None);
        assert_eq!(keyval_to_char(0xFFFFFFFF), None);
    }

    #[test]
    fn test_commands_map_not_empty() {
        // コマンドマップが空でないことを確認
        let commands = ibus_akaza_commands_map();
        assert!(!commands.is_empty(), "Command map should not be empty");
        assert!(
            commands.len() > 20,
            "Command map should have at least 20 commands"
        );
    }

    #[test]
    fn test_commands_map_contains_essential_commands() {
        // 必須コマンドが登録されていることを確認
        let commands = ibus_akaza_commands_map();

        let essential_commands = vec![
            "commit_candidate",
            "commit_preedit",
            "escape",
            "page_up",
            "page_down",
            "set_input_mode_hiragana",
            "set_input_mode_alnum",
            "update_candidates",
            "erase_character_before_cursor",
            "cursor_up",
            "cursor_down",
            "cursor_left",
            "cursor_right",
        ];

        for cmd in essential_commands {
            assert!(
                commands.contains_key(cmd),
                "Command map should contain '{}' command",
                cmd
            );
        }
    }

    #[test]
    fn test_commands_map_contains_number_commands() {
        // 数字キーコマンドが全て登録されていることを確認
        let commands = ibus_akaza_commands_map();

        for i in 0..=9 {
            let cmd_name = format!("press_number_{}", i);
            assert!(
                commands.contains_key(cmd_name.as_str()),
                "Command map should contain '{}' command",
                cmd_name
            );
        }
    }

    #[test]
    fn test_commands_map_contains_conversion_commands() {
        // 変換系コマンドが登録されていることを確認
        let commands = ibus_akaza_commands_map();

        let conversion_commands = vec![
            "convert_to_full_hiragana",
            "convert_to_full_katakana",
            "convert_to_half_katakana",
            "convert_to_full_romaji",
            "convert_to_half_romaji",
        ];

        for cmd in conversion_commands {
            assert!(
                commands.contains_key(cmd),
                "Command map should contain '{}' command",
                cmd
            );
        }
    }

    #[test]
    fn test_commands_map_contains_clause_extension_commands() {
        // 文節伸縮コマンドが登録されていることを確認
        let commands = ibus_akaza_commands_map();

        assert!(commands.contains_key("extend_clause_right"));
        assert!(commands.contains_key("extend_clause_left"));
    }
}
