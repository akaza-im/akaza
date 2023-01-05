use std::collections::HashMap;

use ibus_sys::engine::IBusEngine;

use crate::context::InputMode;
use crate::AkazaContext;

/**
 * shortcut key を設定可能な機能。
 */
pub type IbusAkazaCommand = fn(&mut AkazaContext, *mut IBusEngine);

pub(crate) fn ibus_akaza_commands_map() -> HashMap<&'static str, IbusAkazaCommand> {
    let mut function_map: HashMap<&'static str, IbusAkazaCommand> = HashMap::new();

    // shorthand
    let mut register = |name: &'static str, cmd: IbusAkazaCommand| function_map.insert(name, cmd);

    register("commit_candidate", |context, engine| {
        context.commit_string(engine, context.build_string().as_str());
    });
    // 無変換状態では、ひらがなに変換してコミットします
    register("commit_preedit", |context, engine| {
        let (_, surface) = context.make_preedit_word();
        context.commit_string(engine, surface.as_str());
    });
    register("set_input_mode_hiragana", |context, engine| {
        context.set_input_mode(InputMode::Hiragana, engine)
    });
    register("set_input_mode_alnum", |context, engine| {
        context.set_input_mode(InputMode::Alnum, engine)
    });
    register("update_candidates", |context, engine| {
        context.update_candidates(engine)
    });
    register("erase_character_before_cursor", |context, engine| {
        context.erase_character_before_cursor(engine)
    });
    register("cursor_down", |context, engine| {
        context.cursor_down(engine);
    });
    register("cursor_right", |context, engine| {
        context.cursor_right(engine);
    });

    function_map
}
