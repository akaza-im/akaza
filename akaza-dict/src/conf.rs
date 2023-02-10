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
use gtk4::{CellRendererText, Grid, TreeView, TreeViewColumn};
use log::{info, trace};

use libakaza::config::Config;
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
        let tree_view_column = build_tree_view_column("読み", 0, list_store.clone());
        tree_view.append_column(&tree_view_column);
    }
    {
        let tree_view_column = build_tree_view_column("表記", 1, list_store.clone());
        tree_view.append_column(&tree_view_column);
    }
    // https://gitlab.gnome.org/GNOME/gtk/-/issues/3561
    grid.attach(&tree_view, 0, 0, 6, 1);

    // TODO このへんは Menu にしたい。gtk4-rs で menu を使う方法が分からん。
    let add_button = Button::with_label("追加");
    {
        let list_store = list_store.clone();
        add_button.connect_clicked(move |_| {
            info!("Add new row...");
            list_store.set(&list_store.append(), &[(0, &""), (1, &"")]);
        });
    }
    grid.attach(&add_button, 4, 1, 1, 1);

    let delete_btn = Button::with_label("削除");
    {
        delete_btn.connect_clicked(move |_| {
            let selection = tree_view.selection();
            let Some((a,b)) = selection.selected() else {
                return
            };
            list_store.remove(&b);
            info!("Delete {:?}, {:?}", a, b);
        });
    }
    grid.attach(&delete_btn, 5, 1, 1, 1);

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

fn build_tree_view_column(title: &str, column: u32, list_store: ListStore) -> TreeViewColumn {
    let cell_renderer = CellRendererText::builder()
        .editable(true)
        .xpad(20)
        .ypad(20)
        .build();
    cell_renderer.connect_edited(move |_cell_renderer, _treepath, _str| {
        trace!("{:?}, {:?}", _treepath, _str);
        if _str.is_empty() {
            return;
        }
        let Some(iter) = list_store.iter(&_treepath) else {
            return;
        };
        list_store.set_value(&iter, column, &_str.to_value());
    });
    TreeViewColumn::with_attributes(title, &cell_renderer, &[("text", column as i32)])
}
