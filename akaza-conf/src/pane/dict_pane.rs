use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gtk4::prelude::ButtonExt;
use gtk4::prelude::ComboBoxExt;
use gtk4::prelude::DialogExt;
use gtk4::prelude::EntryBufferExt;
use gtk4::prelude::EntryBufferExtManual;
use gtk4::prelude::FileChooserExt;
use gtk4::prelude::FileExt;
use gtk4::prelude::GridExt;
use gtk4::prelude::GtkWindowExt;
use gtk4::prelude::WidgetExt;
use gtk4::MessageDialog;
use gtk4::{
    Button, ComboBoxText, FileChooserAction, FileChooserDialog, Grid, Label, MessageType,
    ResponseType, ScrolledWindow, Text, TextBuffer, TextView, Window,
};
use log::info;

use libakaza::config::{Config, DictConfig, DictEncoding, DictType, DictUsage};
use libakaza::dict::skk::write::write_skk_dict;

pub fn build_dict_pane(config: Arc<Mutex<Config>>) -> anyhow::Result<ScrolledWindow> {
    let scroll = ScrolledWindow::new();

    let parent_grid = Grid::builder().column_spacing(10).build();
    let grid = Grid::builder().column_spacing(10).build();

    let dicts = config.lock().unwrap().engine.dicts.clone();

    for (i, dict_config) in dicts.iter().enumerate() {
        add_row(&grid, dict_config, &config.clone(), i);
    }

    parent_grid.attach(&grid, 0, 0, 1, 1);

    {
        let add_system_dict_btn = build_add_system_dict_btn(config.clone(), grid.clone());
        parent_grid.attach(&add_system_dict_btn, 0, 1, 1, 1);
    }
    {
        let add_user_dict_btn = build_add_user_dict_btn(grid, config);
        parent_grid.attach(&add_user_dict_btn, 0, 2, 1, 1);
    }
    scroll.set_child(Some(&parent_grid));
    Ok(scroll)
}

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
        for usage in [
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
                    for dict in &mut config.engine.dicts {
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
        for encoding in [DictEncoding::EucJp, DictEncoding::Utf8] {
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
                    for dict in &mut config.engine.dicts {
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

fn build_add_system_dict_btn(config: Arc<Mutex<Config>>, grid: Grid) -> Button {
    let add_btn = Button::with_label("システム辞書の追加");
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
}

fn build_add_user_dict_btn(dict_list_grid: Grid, config: Arc<Mutex<Config>>) -> Button {
    let add_btn = Button::with_label("ユーザー辞書の追加");
    add_btn.connect_clicked(move |_| {
        let window = Window::builder()
            .title("ユーザー辞書の追加")
            .default_width(300)
            .default_height(100)
            .destroy_with_parent(true)
            .modal(true)
            .build();

        let grid = Grid::builder().build();

        let label = TextView::builder()
            .buffer(&TextBuffer::builder().text("辞書名: ").build())
            .build();
        grid.attach(&label, 0, 0, 1, 1);

        let text = Text::builder().build();
        grid.attach(&text, 1, 0, 2, 1);

        let ok_btn = {
            let window = window.clone();
            let ok_btn = Button::with_label("OK");
            let text = text.clone();
            let config = config.clone();
            let dict_list_grid = dict_list_grid.clone();
            ok_btn.set_sensitive(false);
            ok_btn.connect_clicked(move |_| match create_user_dict(&text.buffer().text()) {
                Ok(path) => {
                    let dict_config = DictConfig {
                        path: path.to_string_lossy().to_string(),
                        encoding: DictEncoding::Utf8,
                        dict_type: DictType::SKK,
                        usage: DictUsage::Normal,
                    };
                    let mut locked_conf = config.lock().unwrap();
                    add_row(
                        &dict_list_grid,
                        &dict_config,
                        &config,
                        locked_conf.engine.dicts.len(),
                    );
                    locked_conf.engine.dicts.push(dict_config);
                    window.close();
                }
                Err(err) => {
                    let dialog = MessageDialog::builder()
                        .message_type(MessageType::Error)
                        .text(format!("Error: {err}"))
                        .build();
                    dialog.show();
                }
            });
            grid.attach(&ok_btn, 1, 1, 1, 1);
            ok_btn
        };

        {
            let window = window.clone();
            let cancel_btn = Button::with_label("Cancel");
            cancel_btn.connect_clicked(move |_| {
                window.close();
            });
            grid.attach(&cancel_btn, 2, 1, 1, 1);
        }

        // 辞書名を入力していない場合は OK ボタンを押せない。
        text.buffer().connect_length_notify(move |t| {
            ok_btn.set_sensitive(!t.text().is_empty());
        });

        window.set_child(Some(&grid));
        window.show();
    });
    add_btn
}

fn create_user_dict(name: &str) -> anyhow::Result<PathBuf> {
    let base = xdg::BaseDirectories::with_prefix("akaza")?;

    let userdictdir = base.create_data_directory("userdict")?;
    let path = userdictdir.join(name);
    if !path.as_path().exists() {
        // ファイルがなければカラの SKK 辞書をつくっておく。
        write_skk_dict(&path.to_string_lossy(), vec![])?;
    }

    Ok(path)
}
