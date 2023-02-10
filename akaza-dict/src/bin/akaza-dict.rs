use anyhow::Result;
use log::LevelFilter;

use akaza_dict::conf::open_userdict_window;

use std::env;

/// デバッグ用
fn main() -> Result<()> {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init();

    let args: Vec<String> = env::args().collect();
    open_userdict_window(&args[1])?;
    Ok(())
}
