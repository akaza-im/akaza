use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use gtk::glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Label, Notebook};
use gtk4 as gtk;
use gtk4::gio::ApplicationFlags;
use gtk4::Grid;
use log::{error, info};

use libakaza::config::{Config, EngineConfig};

use crate::pane::{about_pane, core_pane, dict_pane};

pub fn open_configuration_window() -> Result<()> {
    let config = Arc::new(Mutex::new(Config::load()?));
    let app = Application::new(Some("com.github.akaza.config"), ApplicationFlags::empty());

    app.connect_activate(move |app| {
        connect_activate(app, config.clone()).unwrap();
    });

    let v: Vec<String> = Vec::new();
    app.run_with_args(v.as_slice());
    Ok(())
}

fn connect_activate(app: &Application, config: Arc<Mutex<Config>>) -> Result<()> {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(520)
        .default_height(500)
        .title("Akaza の設定")
        .build();

    let notebook = Notebook::builder().vexpand(true).hexpand(true).build();
    notebook.append_page(
        &core_pane::build_core_pane(config.clone())?,
        Some(&Label::new(Some("基本設定"))),
    );
    notebook.append_page(
        &dict_pane::build_dict_pane(config.clone())?,
        Some(&Label::new(Some("辞書"))),
    );
    notebook.append_page(
        &about_pane::build_about_pane(),
        Some(&Label::new(Some("アバウト"))),
    );

    let grid = Grid::builder().build();

    grid.attach(&notebook, 0, 0, 6, 1);

    let ok_button = Button::with_label("OK");
    ok_button.connect_clicked(move |_| {
        eprintln!("Save the configuration...");
        // TODO: 保存処理
        let config = config.lock().unwrap();
        let config = Config {
            keymap: config.keymap.to_string(),
            romkan: config.romkan.to_string(),
            live_conversion: config.live_conversion,
            engine: EngineConfig {
                model: config.engine.model.to_string(),
                dicts: config.engine.dicts.clone(),
                dict_cache: true,
            },
        };
        info!("Saving config: {}", serde_yaml::to_string(&config).unwrap());

        config.save().unwrap();

        // 最後に ibus restart をしちゃおう。設定の再読み込みとか実装するのは大変。
        let output = Command::new("ibus").arg("restart").output().unwrap();

        if !output.status.success() {
            error!(
                "Cannot run `ibus restart`: out={}, err={}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
        } else {
            info!(
                "Ran `ibus restart`: out={}, err={}",
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
        }
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
    Ok(())
}
