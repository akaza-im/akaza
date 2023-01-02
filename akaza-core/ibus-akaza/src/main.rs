mod bindings;
mod wrapper_bindings;

use crate::bindings::{gboolean, guint, ibus_main, IBusEngine};
use crate::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};
use anyhow::Result;
use flexi_logger::{FileSpec, Logger};
use log::info;

unsafe extern "C" fn process_key_event(
    _engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> gboolean {
    info!("process_key_event~~ {}, {}, {}", keyval, keycode, modifiers);
    0
}

fn main() -> Result<()> {
    Logger::try_with_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory("/tmp") // create files in folder ./log_files
                .basename("ibus-akaza")
                .discriminant("Sample4711A") // use infix in log file name
                .suffix("trc"), // use suffix .trc instead of .log
        )
        .print_message() //
        .start()?;

    unsafe {
        ibus_akaza_set_callback(process_key_event);

        ibus_akaza_init(true);

        // run main loop
        ibus_main();
    }
    Ok(())
}
