use std::collections::HashMap;

use anyhow::Result;
use encoding_rs::UTF_8;
use gtk4::builders::MessageDialogBuilder;
use gtk4::gio::ApplicationFlags;
use gtk4::prelude::{
    ApplicationExt, ApplicationExtManual, ButtonExt, EntryBufferExtManual, EntryExt, GridExt,
    GtkApplicationExt, GtkWindowExt, WidgetExt,
};
use gtk4::{
    Application, ApplicationWindow, Button, ButtonsType, Entry, Grid, Inhibit, Label, MessageType,
};
use log::{info, warn};

use libakaza::dict::merge_dict::merge_dict;
use libakaza::dict::skk::read::read_skkdict;
use libakaza::dict::skk::write::write_skk_dict;

pub fn open_dict_register_window() -> Result<()> {
    let app = Application::new(Some("com.github.akaza.dict"), ApplicationFlags::empty());

    app.connect_activate(move |app| {
        connect_activate(app).unwrap();
    });

    let v: Vec<String> = Vec::new();
    app.run_with_args(v.as_slice());
    Ok(())
}

fn connect_activate(app: &Application) -> Result<()> {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(520)
        .default_height(500)
        .title("Akaza の辞書登録")
        .build();

    let grid = Grid::builder().build();

    grid.attach(&Label::builder().label("読み").build(), 0, 0, 1, 1);
    let yomi_entry = Entry::new();
    grid.attach(&yomi_entry, 1, 0, 1, 1);

    grid.attach(&Label::builder().label("漢字").build(), 0, 1, 1, 1);
    let surface_entry = Entry::new();
    grid.attach(&surface_entry, 1, 1, 1, 1);

    let ok_button = Button::with_label("OK");
    {
        let window = window.clone();
        ok_button.connect_clicked(move |_| {
            let yomi = yomi_entry.buffer().text();
            let surface = surface_entry.buffer().text();
            info!("Save new word: {}/{}", surface, yomi);

            let Err(err) = register_word(&yomi, &surface) else {
                yomi_entry.buffer().set_text("");
                surface_entry.buffer().set_text("");

                info!("Close window");
                window.close();

                return;
            };

            let dialog = MessageDialogBuilder::new()
                .buttons(ButtonsType::Close)
                .message_type(MessageType::Info)
                .text(&err.to_string())
                .modal(true)
                .title("辞書保存")
                .build();
            dialog.show();

            yomi_entry.buffer().set_text("");
            surface_entry.buffer().set_text("");
        });
    }
    grid.attach(&ok_button, 0, 2, 1, 1);

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

fn register_word(yomi: &str, surface: &str) -> Result<()> {
    info!("Register new word: {} {}", yomi, surface);

    let basedir = xdg::BaseDirectories::with_prefix("akaza")?;
    let userdict = basedir.create_data_directory("userdict")?;
    let path = userdict.join("userdict.dic");

    let dict = match read_skkdict(path.as_path(), UTF_8) {
        Ok(dict) => dict,
        Err(err) => {
            warn!("Cannot read dict: {}, {}", path.to_string_lossy(), err);
            HashMap::new()
        }
    };

    let dict = merge_dict(vec![
        dict,
        HashMap::from([(yomi.to_string(), vec![surface.to_string()])]),
    ]);

    write_skk_dict(&path.to_string_lossy(), vec![dict])?;

    Ok(())
}
