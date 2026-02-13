use gtk4::Label;

pub fn build_about_pane() -> Label {
    Label::new(Some(
        format!("Akaza version {}", env!("CARGO_PKG_VERSION")).as_str(),
    ))
}
