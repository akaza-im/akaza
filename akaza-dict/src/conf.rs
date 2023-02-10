use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use encoding_rs::UTF_8;
use gtk::glib::signal::Inhibit;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, ListStore};
use gtk4 as gtk;
use gtk4::gio::ApplicationFlags;
use gtk4::glib::Type;
use gtk4::subclass::cell_renderer;
use gtk4::{CellRendererText, Grid, TreeView, TreeViewColumn};
use log::info;

use libakaza::config::Config;
use libakaza::config::DictEncoding::Utf8;
use libakaza::dict::skk::read::read_skkdict;

pub fn open_userdict_window(user_dict_path: &str) -> Result<()> {
    let config = Arc::new(Mutex::new(Config::load()?));
    let app = Application::new(Some("com.github.akaza.config"), ApplicationFlags::empty());

    let user_dict_path = user_dict_path.to_string();
    app.connect_activate(move |app| {
        connect_activate(app, config.clone(), &user_dict_path).unwrap();
    });

    let v: Vec<String> = Vec::new();
    app.run_with_args(v.as_slice());
    Ok(())
}

fn connect_activate(
    app: &Application,
    config: Arc<Mutex<Config>>,
    user_dict_path: &str,
) -> Result<()> {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(520)
        .default_height(500)
        .title("Akaza の設定")
        .build();

    let grid = Grid::builder().build();

    info!("Loading skk dict from {user_dict_path}");
    let dict = read_skkdict(Path::new(user_dict_path), UTF_8)?;
    let dict = dict
        .iter()
        .flat_map(|(yomi, surfaces)| {
            surfaces
                .iter()
                .map(|surface| (yomi.to_string(), surface.to_string()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let list_store = ListStore::new(&[Type::STRING, Type::STRING]);
    for (yomi, surface) in dict {
        list_store.set(&list_store.append(), &[(0, &yomi), (1, &surface)]);
    }
    // list_store.set(&list_store.append(), &[(0, &"world".to_string())]);
    let tree_view = TreeView::builder().model(&list_store).build();
    {
        let tree_view_column =
            TreeViewColumn::with_attributes("読み", &CellRendererText::new(), &[("text", 0)]);
        tree_view.append_column(&tree_view_column);
    }
    {
        let tree_view_column =
            TreeViewColumn::with_attributes("表記", &CellRendererText::new(), &[("text", 1)]);
        tree_view.append_column(&tree_view_column);
    }
    grid.attach(&tree_view, 0, 0, 6, 1);

    let add_button = Button::with_label("追加");
    add_button.connect_clicked(move |_| {
        eprintln!("Save the configuration...");
        // TODO: 保存処理
    });

    let delete_button = Button::with_label("Cancel");
    {
        let window_clone = window.clone();
        delete_button.connect_clicked(move |_| {
            eprintln!("Close the configuration window!");
            window_clone.close();
        });
    }
    grid.attach(&add_button, 4, 1, 1, 1);
    grid.attach(&delete_button, 5, 1, 1, 1);

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
