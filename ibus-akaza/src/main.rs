#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{thread, time};

use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};

use ibus_sys::core::ibus_main;
use ibus_sys::engine::IBusEngine;
use ibus_sys::glib::guint;
use libakaza::engine::akaza_builder::BigramWordViterbiEngineBuilder;
use libakaza::user_side_data::user_data::UserData;

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
    context_ref.process_key_event(engine, keyval, keycode, modifiers)
}

unsafe extern "C" fn candidate_clicked(
    context: *mut c_void,
    engine: *mut IBusEngine,
    index: guint,
    button: guint,
    state: guint,
) {
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.do_candidate_clicked(engine, index, button, state);
}

fn load_user_data() -> Arc<Mutex<UserData>> {
    match UserData::load_from_default_path() {
        Ok(user_data) => Arc::new(Mutex::new(user_data)),
        Err(err) => {
            error!("Cannot load user data: {}", err);
            Arc::new(Mutex::new(UserData::default()))
        }
    }
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct IBusAkazaArgs {
    #[clap(long)]
    ibus: bool,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    let arg: IBusAkazaArgs = IBusAkazaArgs::parse();

    env_logger::Builder::new()
        .filter_level(arg.verbose.log_level_filter())
        .init();

    info!("Starting ibus-akaza(rust version)");

    unsafe {
        let sys_time = SystemTime::now();
        let user_data = load_user_data();
        let akaza = BigramWordViterbiEngineBuilder::default()
            .user_data(user_data.clone())
            .system_data_dir("/home/tokuhirom/dev/akaza/akaza-data/data")
            .build()?;
        let mut ac = AkazaContext::new(akaza);
        let new_sys_time = SystemTime::now();
        let difference = new_sys_time.duration_since(sys_time)?;
        info!(
            "Initialized ibus-akaza engine in {} milliseconds.",
            difference.as_millis()
        );

        thread::Builder::new()
            .name("user-data-save-thread".to_string())
            .spawn(move || {
                let interval = time::Duration::from_secs(3);

                // スレッド内で雑に例外投げるとスレッドとまっちゃうので丁寧めに処理する。
                loop {
                    if let Ok(data) = user_data.lock() {
                        if let Err(e) = data.write_user_stats_file() {
                            warn!("Cannot save user stats file: {}", e);
                        }
                    } else {
                        warn!("Cannot get mutex for saving user data")
                    };
                    thread::sleep(interval);
                }
            })?;

        ibus_akaza_set_callback(
            &mut ac as *mut _ as *mut c_void,
            process_key_event,
            candidate_clicked,
        );

        ibus_akaza_init(arg.ibus);

        info!("Enter the ibus_main()");

        // run main loop
        ibus_main();

        warn!("Should not reach here.");
    }
    Ok(())
}
