use akaza_conf::conf::open_configuration_window;
use anyhow::Result;
use log::LevelFilter;

/// デバッグ用
fn main() -> Result<()> {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init();

    open_configuration_window()?;
    Ok(())
}
