use std::collections::HashMap;

use ibus_sys::bindings::IBusEngine;

use crate::context::InputMode;
use crate::{AkazaContext, _make_preedit_word};

pub type IbusAkazaCommand = fn(&mut AkazaContext, *mut IBusEngine);

macro_rules! command {
    ($i: ident) => {
        (stringify!($i), $i as IbusAkazaCommand)
    };
}
pub(crate) fn ibus_akaza_commands_map() -> HashMap<&'static str, IbusAkazaCommand> {
    HashMap::from([
        command!(commit_candidate),
        command!(commit_preedit),
        command!(erase_character_before_cursor),
        command!(set_input_mode_hiragana),
        command!(set_input_mode_alnum),
        command!(update_candidates),
    ])
}

fn commit_candidate(context: &mut AkazaContext, engine: *mut IBusEngine) {
    let s = context.build_string();
    context.commit_string(engine, s.as_str());
}

fn commit_preedit(context: &mut AkazaContext, engine: *mut IBusEngine) {
    /*
    # 無変換状態では、ひらがなに変換してコミットします。
    yomi, word = self._make_preedit_word()
    self.commit_string(word)
     */
    unsafe {
        let (_, surface) = _make_preedit_word(context);
        context.commit_string(engine, surface.as_str());
    }
}

fn set_input_mode_hiragana(context: &mut AkazaContext, engine: *mut IBusEngine) {
    context.set_input_mode(InputMode::Hiragana, engine)
}

fn set_input_mode_alnum(context: &mut AkazaContext, engine: *mut IBusEngine) {
    context.set_input_mode(InputMode::Alnum, engine)
}

fn update_candidates(context: &mut AkazaContext, engine: *mut IBusEngine) {
    context.update_candidates(engine)
}

fn erase_character_before_cursor(context: &mut AkazaContext, engine: *mut IBusEngine) {
    context.erase_character_before_cursor(engine);
}
