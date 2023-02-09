use gtk4::prelude::{CheckButtonExt, ComboBoxExt, GridExt};
use gtk4::{CheckButton, ComboBoxText, Grid, Label};
use libakaza::config::Config;
use log::info;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn build_core_pane(config: Arc<Mutex<Config>>) -> anyhow::Result<Grid> {
    // キーマップとローマ字テーブル、モデルの設定ができるようにする。
    let grid = Grid::new();
    // xalign: 0 は左寄という意味。
    grid.attach(
        &Label::builder().label("キーマップ").xalign(0_f32).build(),
        0,
        0,
        1,
        1,
    );
    grid.attach(
        &{
            let cbt = ComboBoxText::new();
            let keymap = get_list("keymap", |f| f.to_string_lossy().ends_with(".yml"));
            for item in keymap {
                cbt.append(Some(&item.path), &item.name);
            }
            cbt.set_active_id(Some(&config.lock().unwrap().keymap));
            {
                let config = config.clone();
                cbt.connect_changed(move |f| {
                    if let Some(id) = f.active_id() {
                        config.lock().unwrap().keymap = id.to_string();
                    }
                });
            }
            cbt
        },
        1,
        0,
        1,
        1,
    );
    grid.attach(
        &Label::builder()
            .label("ローマ字テーブル")
            .xalign(0_f32)
            .build(),
        0,
        1,
        1,
        1,
    );
    grid.attach(
        &{
            let cbt = ComboBoxText::new();
            let romkan = get_list("romkan", |f| f.to_string_lossy().ends_with(".yml"));
            info!("romkan: {:?}", romkan);
            for item in romkan {
                cbt.append(Some(&item.path), &item.name);
            }
            cbt.set_active_id(Some(&config.lock().unwrap().romkan));

            let config = config.clone();
            cbt.connect_changed(move |f| {
                if let Some(id) = f.active_id() {
                    config.lock().unwrap().romkan = id.to_string();
                }
            });

            cbt
        },
        1,
        1,
        1,
        1,
    );
    grid.attach(
        &Label::builder().label("言語モデル").xalign(0_f32).build(),
        0,
        2,
        1,
        1,
    );
    grid.attach(
        &{
            let cbt = ComboBoxText::new();
            let model = get_list("model", {
                |f| !f.file_name().unwrap().to_string_lossy().starts_with('.')
            });
            info!("model: {:?}", model);
            for item in model {
                cbt.append(Some(&item.path), &item.name);
            }
            cbt.set_active_id(Some(&config.lock().unwrap().engine.model));

            let config = config.clone();
            cbt.connect_changed(move |f| {
                if let Some(id) = f.active_id() {
                    config.lock().unwrap().engine.model = id.to_string();
                }
            });

            cbt
        },
        1,
        2,
        1,
        1,
    );
    {
        let check_box = CheckButton::builder()
            .label("ライブ変換")
            .active(config.lock().unwrap().live_conversion)
            .build();
        grid.attach(&check_box, 0, 3, 1, 1);
        check_box.connect_toggled(move |f| {
            config.lock().unwrap().live_conversion = f.is_active();
        });
    }
    Ok(grid)
}

pub(crate) fn get_list<P>(path: &str, filter: P) -> Vec<PathConfItem>
where
    P: FnMut(&&PathBuf) -> bool,
{
    let p = xdg::BaseDirectories::with_prefix("akaza")
        .unwrap()
        .list_data_files(path);

    p.iter()
        .filter(filter)
        .map(|f| PathConfItem {
            name: f
                .as_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .replace(".yml", ""),
            path: f.to_string_lossy().to_string(),
        })
        .collect::<Vec<_>>()
}

#[derive(Debug)]
pub(crate) struct PathConfItem {
    name: String,
    path: String,
}
