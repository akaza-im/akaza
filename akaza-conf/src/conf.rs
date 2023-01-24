use std::path::PathBuf;

use gtk::glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Label, Notebook};
use gtk4 as gtk;
use gtk4::builders::ComboBoxTextBuilder;
use gtk4::gio::ApplicationFlags;
use gtk4::{ComboBoxText, Grid};
use libakaza::config::Config;
use log::info;

pub fn open_configuration_window() {
    let app = Application::new(Some("com.github.akaza.config"), ApplicationFlags::empty());

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("Akaza の設定")
            .build();

        let notebook = Notebook::builder().build();
        notebook.append_page(&build_core_pane(), Some(&Label::new(Some("基本設定"))));
        notebook.append_page(&build_dict_pane(), Some(&Label::new(Some("辞書"))));
        notebook.append_page(&build_about_pane(), Some(&Label::new(Some("アバウト"))));

        let grid = Grid::builder().build();

        grid.attach(&notebook, 0, 0, 6, 1);

        let ok_button = Button::with_label("OK");
        ok_button.connect_clicked(|_| {
            eprintln!("Save the configuration...");
            // TODO: 保存処理
        });
        let cancel_button = Button::with_label("Cancel");
        {
            let window_clone = window.clone();
            cancel_button.connect_clicked(move |_| {
                eprintln!("Close the configuration window!");
                window_clone.close();
            });
        }
        grid.attach(&ok_button, 4, 1, 1, 1);
        grid.attach(&cancel_button, 5, 1, 1, 1);

        window.set_child(Some(&grid));

        window.connect_close_request(move |window| {
            if let Some(application) = window.application() {
                application.remove_window(window);
            }
            Inhibit(false)
        });

        window.show();
    });

    let v: Vec<String> = Vec::new();
    app.run_with_args(v.as_slice());
}

#[derive(Debug)]
struct PathConfItem {
    name: String,
    path: String,
}

fn get_keymap_list<P>(path: &str, filter: P) -> Vec<PathConfItem>
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
                .to_string(),
            path: f.to_string_lossy().to_string(),
        })
        .collect::<Vec<_>>()
}

fn build_core_pane() -> Grid {
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
            let romkan = get_keymap_list("keymap", { |f| f.to_string_lossy().ends_with(".yml") });
            info!("keymap: {:?}", romkan);
            for item in romkan {
                cbt.append(Some(&item.path), &item.name);
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
            let romkan = get_keymap_list("romkan", { |f| f.to_string_lossy().ends_with(".yml") });
            info!("romkan: {:?}", romkan);
            for item in romkan {
                cbt.append(Some(&item.path), &item.name);
            }
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
            let romkan = get_keymap_list("model", {
                |f| !f.file_name().unwrap().to_string_lossy().starts_with(".")
            });
            info!("model: {:?}", romkan);
            for item in romkan {
                cbt.append(Some(&item.path), &item.name);
            }
            cbt
        },
        1,
        2,
        1,
        1,
    );
    grid
}

fn build_dict_pane() -> Label {
    Label::new(Some("(工事中)"))
}

fn build_about_pane() -> Label {
    Label::new(Some(
        format!("Akaza version {}", env!("CARGO_PKG_VERSION")).as_str(),
    ))
}
