use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use ibus_sys::core::to_gboolean;
use ibus_sys::engine::{ibus_engine_register_properties, ibus_engine_update_property, IBusEngine};
use ibus_sys::glib::{g_object_ref_sink, gchar, gpointer};
use ibus_sys::prop_list::{ibus_prop_list_append, ibus_prop_list_new, IBusPropList};
use ibus_sys::property::{
    ibus_property_new, ibus_property_set_label, ibus_property_set_state,
    ibus_property_set_sub_props, ibus_property_set_symbol, IBusPropState_PROP_STATE_CHECKED,
    IBusPropState_PROP_STATE_UNCHECKED, IBusPropType_PROP_TYPE_MENU, IBusPropType_PROP_TYPE_RADIO,
    IBusProperty,
};
use ibus_sys::text::{IBusText, StringExt};
use libakaza::config::{Config, DictConfig};

use crate::input_mode::{get_all_input_modes, InputMode};

pub struct PropController {
    prop_list: *mut IBusPropList,
    /// input mode のメニューの親プロパティ。
    input_mode_prop: *mut IBusProperty,
    /// メニューの input mode ごとのメニュープロパティたち。
    prop_dict: HashMap<String, *mut IBusProperty>,
}

impl PropController {
    pub fn new(initial_input_mode: InputMode, config: Config) -> Result<Self> {
        let (input_mode_prop, prop_list, prop_dict) = Self::init_props(initial_input_mode, config)?;

        Ok(PropController {
            prop_list,
            input_mode_prop,
            prop_dict,
        })
    }

    /// ibus の do_focus_in のときに呼ばれる。
    pub fn do_focus_in(&self, engine: *mut IBusEngine) {
        unsafe {
            ibus_engine_register_properties(engine, self.prop_list);
        }
    }

    /// タスクメニューからポップアップして選べるメニューを構築する。
    ///
    /// * `initial_input_mode`: 初期状態の input_mode
    fn init_props(
        initial_input_mode: InputMode,
        config: Config,
    ) -> Result<(
        *mut IBusProperty,
        *mut IBusPropList,
        HashMap<String, *mut IBusProperty>,
    )> {
        unsafe {
            let prop_list =
                g_object_ref_sink(ibus_prop_list_new() as gpointer) as *mut IBusPropList;

            let input_mode_prop = g_object_ref_sink(ibus_property_new(
                "InputMode\0".as_ptr() as *const gchar,
                IBusPropType_PROP_TYPE_MENU,
                format!("入力モード: {}", initial_input_mode.symbol).to_ibus_text(),
                "\0".as_ptr() as *const gchar,
                "Switch input mode".to_ibus_text(),
                to_gboolean(true),
                to_gboolean(true),
                IBusPropState_PROP_STATE_UNCHECKED,
                std::ptr::null_mut() as *mut IBusPropList,
            ) as gpointer) as *mut IBusProperty;
            ibus_prop_list_append(prop_list, input_mode_prop);

            let props = g_object_ref_sink(ibus_prop_list_new() as gpointer) as *mut IBusPropList;
            let mut prop_map: HashMap<String, *mut IBusProperty> = HashMap::new();
            for input_mode in get_all_input_modes() {
                let prop = g_object_ref_sink(ibus_property_new(
                    (input_mode.prop_name.to_string() + "\0").as_ptr() as *const gchar,
                    IBusPropType_PROP_TYPE_RADIO,
                    input_mode.label.to_ibus_text(),
                    "\0".as_ptr() as *const gchar,
                    std::ptr::null_mut() as *mut IBusText,
                    to_gboolean(true),
                    to_gboolean(true),
                    if input_mode.mode_code == initial_input_mode.mode_code {
                        IBusPropState_PROP_STATE_CHECKED
                    } else {
                        IBusPropState_PROP_STATE_UNCHECKED
                    },
                    std::ptr::null_mut() as *mut IBusPropList,
                ) as gpointer) as *mut IBusProperty;
                prop_map.insert(input_mode.prop_name.to_string(), prop);
                ibus_prop_list_append(props, prop);
            }
            ibus_property_set_sub_props(input_mode_prop, props);

            // ユーザー辞書
            Self::build_user_dict(prop_list, config)?;

            // 設定ファイルを開くというやつ
            Self::build_preference_menu(prop_list);

            Ok((input_mode_prop, prop_list, prop_map))
        }
    }

    unsafe fn build_user_dict(prop_list: *mut IBusPropList, config: Config) -> Result<()> {
        let user_dict_prop = g_object_ref_sink(ibus_property_new(
            "UserDict\0".as_ptr() as *const gchar,
            IBusPropType_PROP_TYPE_MENU,
            "ユーザー辞書".to_ibus_text(),
            "\0".as_ptr() as *const gchar,
            "User dict".to_ibus_text(),
            to_gboolean(true),
            to_gboolean(true),
            IBusPropState_PROP_STATE_UNCHECKED,
            std::ptr::null_mut() as *mut IBusPropList,
        ) as gpointer) as *mut IBusProperty;
        ibus_prop_list_append(prop_list, user_dict_prop);

        let props = g_object_ref_sink(ibus_prop_list_new() as gpointer) as *mut IBusPropList;
        for dict in Self::find_user_dicts(config)? {
            let prop = g_object_ref_sink(ibus_property_new(
                ("UserDict.".to_string() + dict.path.as_str() + "\0").as_ptr() as *const gchar,
                IBusPropType_PROP_TYPE_MENU,
                Path::new(&dict.path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_ibus_text(),
                "\0".as_ptr() as *const gchar,
                std::ptr::null_mut() as *mut IBusText,
                to_gboolean(true),
                to_gboolean(true),
                IBusPropState_PROP_STATE_UNCHECKED,
                std::ptr::null_mut() as *mut IBusPropList,
            ) as gpointer) as *mut IBusProperty;
            // prop_map.insert(input_mode.prop_name.to_string(), prop);
            ibus_prop_list_append(props, prop);
        }
        ibus_property_set_sub_props(user_dict_prop, props);
        Ok(())
    }

    fn find_user_dicts(config: Config) -> anyhow::Result<Vec<DictConfig>> {
        let dir = xdg::BaseDirectories::with_prefix("akaza")?;
        let dir = dir.create_data_directory("userdict")?;
        let dicts = config
            .engine
            .dicts
            .iter()
            .filter(|f| f.path.contains(&dir.to_string_lossy().to_string()))
            .cloned()
            .collect::<Vec<_>>();

        Ok(dicts)
    }

    unsafe fn build_preference_menu(prop_list: *mut IBusPropList) {
        let preference_prop = g_object_ref_sink(ibus_property_new(
            "PrefPane\0".as_ptr() as *const gchar,
            IBusPropType_PROP_TYPE_MENU,
            "設定".to_ibus_text(),
            "\0".as_ptr() as *const gchar,
            "Preference".to_ibus_text(),
            to_gboolean(true),
            to_gboolean(true),
            IBusPropState_PROP_STATE_UNCHECKED,
            std::ptr::null_mut() as *mut IBusPropList,
        ) as gpointer) as *mut IBusProperty;
        ibus_prop_list_append(prop_list, preference_prop);
    }

    /// input_mode の切り替え時に実行される処理
    pub fn set_input_mode(&self, input_mode: &InputMode, engine: *mut IBusEngine) {
        // メニューの親項目のラベルを変更したい。
        unsafe {
            ibus_property_set_symbol(self.input_mode_prop, input_mode.symbol.to_ibus_text());
            ibus_property_set_label(
                self.input_mode_prop,
                format!("入力モード: {}", input_mode.symbol).to_ibus_text(),
            );
            ibus_engine_update_property(engine, self.input_mode_prop);
        }

        // 有効化する input mode のメニュー項目にチェックを入れる。
        let Some(property) = self.prop_dict.get(input_mode.prop_name) else {
            panic!("Unknown input mode: {input_mode:?}");
        };
        unsafe {
            ibus_property_set_state(*property, IBusPropState_PROP_STATE_CHECKED);
            ibus_engine_update_property(engine, *property);
        }
    }
}
