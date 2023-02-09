use gtk4::prelude::{
    ButtonExt, ComboBoxExt, DialogExt, FileChooserExt, FileExt, GridExt, GtkWindowExt, WidgetExt,
};
use gtk4::{
    Button, ComboBoxText, FileChooserAction, FileChooserDialog, Grid, Label, ResponseType,
    ScrolledWindow, Window,
};
use libakaza::config::{Config, DictConfig, DictEncoding, DictType, DictUsage};
use log::info;
use std::sync::{Arc, Mutex};

pub fn build_dict_pane(config: Arc<Mutex<Config>>) -> anyhow::Result<ScrolledWindow> {
    // TODO ここは TreeView 使った方がすっきり書けるはずだが、僕の GTK+ 力が引くすぎて対応できていない。
    // 誰かすっきり使い易くしてほしい。
    fn add_row(grid: &Grid, dict_config: &DictConfig, config: &Arc<Mutex<Config>>, i: usize) {
        grid.attach(
            &Label::builder()
                .xalign(0_f32)
                .label(dict_config.path.as_str())
                .build(),
            0,
            i as i32,
            1,
            1,
        );

        {
            let cbt = ComboBoxText::builder().build();
            for usage in vec![
                DictUsage::Normal,
                DictUsage::SingleTerm,
                DictUsage::Disabled,
            ] {
                cbt.append(Some(usage.as_str()), usage.text_jp());
            }
            cbt.set_active_id(Some(dict_config.usage.as_str()));
            {
                let config = config.clone();
                let path = dict_config.path.clone();
                cbt.connect_changed(move |f| {
                    if let Some(id) = f.active_id() {
                        let mut config = config.lock().unwrap();
                        for mut dict in &mut config.engine.dicts {
                            if dict.path == path {
                                dict.usage = DictUsage::from(&id).unwrap();
                                return;
                            }
                        }
                        config.engine.dicts.push(DictConfig {
                            dict_type: DictType::SKK,
                            encoding: DictEncoding::EucJp,
                            path: path.to_string(),
                            usage: DictUsage::from(&id).unwrap(),
                        })
                    }
                });
            }
            grid.attach(&cbt, 1, i as i32, 1, 1);
        }

        grid.attach(
            &Label::new(Some(dict_config.dict_type.as_str())),
            2,
            i as i32,
            1,
            1,
        );
        {
            let cbt = ComboBoxText::builder().build();
            for encoding in vec![DictEncoding::EucJp, DictEncoding::Utf8] {
                cbt.append(
                    Some(&encoding.to_string()),
                    encoding.as_str().replace('_', "-").as_str(),
                );
            }
            cbt.set_active_id(Some(dict_config.encoding.as_str()));
            {
                let config = config.clone();
                let path = dict_config.path.clone();
                cbt.connect_changed(move |f| {
                    if let Some(id) = f.active_id() {
                        let mut config = config.lock().unwrap();
                        for mut dict in &mut config.engine.dicts {
                            if dict.path == path {
                                dict.encoding = DictEncoding::from(&id).unwrap();
                                break;
                            }
                        }
                    }
                });
            }
            grid.attach(&cbt, 3, i as i32, 1, 1);
        }

        {
            let delete_btn = {
                let path = dict_config.path.clone();
                let config = config.clone();
                let delete_btn = Button::with_label("削除");
                let grid = grid.clone();
                delete_btn.connect_clicked(move |_| {
                    let mut config = config.lock().unwrap();
                    for (i, dict) in &mut config.engine.dicts.iter().enumerate() {
                        if dict.path == path {
                            config.engine.dicts.remove(i);
                            grid.remove_row(i as i32);
                            break;
                        }
                    }
                });
                delete_btn
            };
            grid.attach(&delete_btn, 4, i as i32, 1, 1);
        }
    }

    let scroll = ScrolledWindow::new();

    let parent_grid = Grid::builder().column_spacing(10).build();
    let grid = Grid::builder().column_spacing(10).build();

    let dicts = config.lock().unwrap().engine.dicts.clone();

    for (i, dict_config) in dicts.iter().enumerate() {
        add_row(&grid, dict_config, &config.clone(), i);
    }

    parent_grid.attach(&grid, 0, 0, 1, 1);

    {
        let add_btn = {
            let add_btn = Button::with_label("Add");
            let config = config;
            let grid = grid;
            add_btn.connect_clicked(move |_| {
                let dialog = FileChooserDialog::new(
                    Some("辞書の選択"),
                    None::<&Window>,
                    FileChooserAction::Open,
                    &[
                        ("開く", ResponseType::Accept),
                        ("キャンセル", ResponseType::None),
                    ],
                );
                let config = config.clone();
                let grid = grid.clone();
                dialog.connect_response(move |dialog, resp| match resp {
                    ResponseType::Accept => {
                        let file = dialog.file().unwrap();
                        let path = file.path().unwrap();

                        info!("File: {:?}", path);
                        let dict_config = &DictConfig {
                            path: path.to_string_lossy().to_string(),
                            encoding: DictEncoding::Utf8,
                            usage: DictUsage::Normal,
                            dict_type: DictType::SKK,
                        };
                        config
                            .lock()
                            .unwrap()
                            .engine
                            .dicts
                            .push(dict_config.clone());
                        add_row(
                            &grid,
                            dict_config,
                            &config.clone(),
                            config.lock().unwrap().engine.dicts.len(),
                        );
                        dialog.close();
                    }
                    ResponseType::Close
                    | ResponseType::Reject
                    | ResponseType::Yes
                    | ResponseType::No
                    | ResponseType::None
                    | ResponseType::DeleteEvent => {
                        dialog.close();
                    }
                    _ => {}
                });
                dialog.show();
            });
            add_btn
        };
        parent_grid.attach(&add_btn, 0, 1, 1, 1);
    }
    scroll.set_child(Some(&parent_grid));
    Ok(scroll)
}
