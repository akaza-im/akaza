#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::time::SystemTime;

use anyhow::Result;
use log::{info, warn};

use ibus_sys::core::ibus_main;
use ibus_sys::engine::IBusEngine;
use ibus_sys::glib::guint;
use libakaza::akaza_builder::AkazaBuilder;

use crate::context::AkazaContext;
use crate::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};

mod commands;
mod context;
mod keymap;
mod wrapper_bindings;

unsafe extern "C" fn process_key_event(
    context: *mut c_void,
    engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> bool {
    let context_ref = &mut *(context as *mut AkazaContext);
    return context_ref.process_key_event(engine, keyval, keycode, modifiers);
}

fn main() -> Result<()> {
    env_logger::init();

    info!("Starting ibus-akaza(rust version)");

    unsafe {
        let sys_time = SystemTime::now();
        let akaza = AkazaBuilder::default()
            // TODO take dictionary path from command line option.
            .system_data_dir("/home/tokuhirom/dev/akaza/akaza-data/data")
            .build()?;
        let mut ac = AkazaContext::new(akaza);
        let new_sys_time = SystemTime::now();
        let difference = new_sys_time.duration_since(sys_time)?;
        info!(
            "Initialized ibus-akaza engine in {} milliseconds.",
            difference.as_millis()
        );

        ibus_akaza_set_callback(&mut ac as *mut _ as *mut c_void, process_key_event);

        ibus_akaza_init(true);

        info!("Enter the ibus_main()");

        // run main loop
        ibus_main();

        warn!("Should not reach here.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
