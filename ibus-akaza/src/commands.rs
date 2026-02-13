use std::collections::HashMap;

use ibus_sys::engine::IBusEngine;

use log::error;

use crate::context::AkazaContext;
use crate::input_mode::{
    INPUT_MODE_ALNUM, INPUT_MODE_FULLWIDTH_ALNUM, INPUT_MODE_HALFWIDTH_KATAKANA,
    INPUT_MODE_HIRAGANA, INPUT_MODE_KATAKANA,
};

/**
 * shortcut key を設定可能な機能。
 */
pub type IbusAkazaCommand = fn(&mut AkazaContext, *mut IBusEngine) -> bool;

pub fn ibus_akaza_commands_map() -> HashMap<&'static str, IbusAkazaCommand> {
    let mut function_map: HashMap<&'static str, IbusAkazaCommand> = HashMap::new();

    // shorthand
    let mut register = |name: &'static str, cmd: IbusAkazaCommand| function_map.insert(name, cmd);

    register("commit_candidate", |context, engine| {
        context.commit_candidate(engine);
        true
    });
    // 無変換状態では、ひらがなに変換してコミットします
    register("commit_preedit", |context, engine| {
        context.commit_preedit(engine);
        true
    });
    register("escape", |context, engine| {
        context.escape(engine);
        true
    });
    register("page_up", |context, engine| {
        context.page_up(engine);
        true
    });
    register("page_down", |context, engine| {
        context.page_down(engine);
        true
    });

    register("set_input_mode_hiragana", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_HIRAGANA);
        true
    });
    register("set_input_mode_alnum", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_ALNUM);
        true
    });
    register("set_input_mode_fullwidth_alnum", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_FULLWIDTH_ALNUM);
        true
    });
    register("set_input_mode_katakana", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_KATAKANA);
        true
    });
    register("set_input_mode_halfwidth_katakana", |context, engine| {
        context.set_input_mode(engine, &INPUT_MODE_HALFWIDTH_KATAKANA);
        true
    });

    register("update_candidates", |context, engine| {
        context.update_candidates(engine)
    });
    register("erase_character_before_cursor", |context, engine| {
        context.erase_character_before_cursor(engine);
        true
    });
    register("cursor_up", |context, engine| context.cursor_up(engine));
    register("cursor_down", |context, engine| context.cursor_down(engine));
    register("cursor_right", |context, engine| {
        context.cursor_right(engine);
        true
    });
    register("cursor_left", |context, engine| {
        context.cursor_left(engine);
        true
    });
    register("extend_clause_right", |context, engine| {
        if let Err(e) = context.extend_clause_right(engine) {
            error!("extend_clause_right failed: {}", e);
            return false;
        }
        true
    });
    register("extend_clause_left", |context, engine| {
        if let Err(e) = context.extend_clause_left(engine) {
            error!("extend_clause_left failed: {}", e);
            return false;
        }
        true
    });
    register("cycle_segmentation", |context, engine| {
        context.cycle_segmentation(engine)
    });
    register("convert_to_full_hiragana", |context, engine| {
        if let Err(e) = context.convert_to_full_hiragana(engine) {
            error!("convert_to_full_hiragana failed: {}", e);
            return false;
        }
        true
    });
    register("convert_to_full_katakana", |context, engine| {
        if let Err(e) = context.convert_to_full_katakana(engine) {
            error!("convert_to_full_katakana failed: {}", e);
            return false;
        }
        true
    });
    register("convert_to_half_katakana", |context, engine| {
        if let Err(e) = context.convert_to_half_katakana(engine) {
            error!("convert_to_half_katakana failed: {}", e);
            return false;
        }
        true
    });
    register("convert_to_full_romaji", |context, engine| {
        if let Err(e) = context.convert_to_full_romaji(engine) {
            error!("convert_to_full_romaji failed: {}", e);
            return false;
        }
        true
    });
    register("convert_to_half_romaji", |context, engine| {
        if let Err(e) = context.convert_to_half_romaji(engine) {
            error!("convert_to_half_romaji failed: {}", e);
            return false;
        }
        true
    });

    {
        // TODO コピペがすごい。マクロかうまいなにかでまとめて登録できるようにしたい。
        register("press_number_1", |context, engine| {
            context.process_num_key(1, engine)
        });
        register("press_number_2", |context, engine| {
            context.process_num_key(2, engine)
        });
        register("press_number_3", |context, engine| {
            context.process_num_key(3, engine)
        });
        register("press_number_4", |context, engine| {
            context.process_num_key(4, engine)
        });
        register("press_number_5", |context, engine| {
            context.process_num_key(5, engine)
        });
        register("press_number_6", |context, engine| {
            context.process_num_key(6, engine)
        });
        register("press_number_7", |context, engine| {
            context.process_num_key(7, engine)
        });
        register("press_number_8", |context, engine| {
            context.process_num_key(8, engine)
        });
        register("press_number_9", |context, engine| {
            context.process_num_key(9, engine)
        });
        register("press_number_0", |context, engine| {
            context.process_num_key(0, engine)
        });
    }

    function_map
}
