use anyhow::Result;
use std::fs::File;

pub type IBusBus = [u64; 6usize];

extern "C" {
    pub fn ibus_bus_new() -> *mut IBusBus;
    pub fn ibus_init();
    pub fn ibus_main();

    /// is_ibus: true if the project run with `--ibus` option.
    pub fn ibus_akaza_init(is_ibus: bool);

}

fn main() -> Result<()> {
    unsafe {
        File::create("/tmp/ibus-akaza-started.log")?;

        ibus_akaza_init(true);

        // run main loop
        ibus_main();
    }
    Ok(())
}
