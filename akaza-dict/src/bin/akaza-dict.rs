use akaza_dict::dict::open_dict_register_window;
use anyhow::Result;
use log::LevelFilter;

fn main() -> Result<()> {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init();

    open_dict_register_window()?;

    Ok(())
}
