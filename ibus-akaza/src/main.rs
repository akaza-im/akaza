#![allow(non_upper_case_globals)]

use std::ffi::{c_char, c_void, CStr};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{fs, fs::OpenOptions, thread, time};

use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};

use ibus_sys::core::ibus_main;
use ibus_sys::engine::IBusEngine;
use ibus_sys::glib::{gchar, guint};
use libakaza::config::Config;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;
use libakaza::user_side_data::user_data::UserData;

use ibus_akaza_lib::context::AkazaContext;
use ibus_akaza_lib::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};

unsafe extern "C" fn process_key_event(
    context: *mut c_void,
    engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> bool {
    if context.is_null() {
        error!("process_key_event: context is null");
        return false;
    }
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
    if context.is_null() {
        error!("candidate_clicked: context is null");
        return;
    }
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.do_candidate_clicked(engine, index, button, state);
}

unsafe extern "C" fn focus_in(context: *mut c_void, engine: *mut IBusEngine) {
    if context.is_null() {
        error!("focus_in: context is null");
        return;
    }
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.do_focus_in(engine);
}

unsafe extern "C" fn property_activate(
    context: *mut c_void,
    engine: *mut IBusEngine,
    prop_name: *mut gchar,
    prop_state: guint,
) {
    if context.is_null() {
        error!("property_activate: context is null");
        return;
    }
    if prop_name.is_null() {
        error!("property_activate: prop_name is null");
        return;
    }
    let prop_name_str = CStr::from_ptr(prop_name as *mut c_char)
        .to_string_lossy()
        .to_string();
    info!(
        "property_activate callback fired: prop_name={:?}, prop_state={}",
        prop_name_str, prop_state
    );
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.do_property_activate(engine, prop_name_str, prop_state);
}

/// 暗号化鍵を読み込む。なければ生成して保存する。
fn load_or_create_encryption_key() -> Result<Vec<u8>> {
    let basedir = xdg::BaseDirectories::with_prefix("akaza")?;
    let key_path = basedir.place_data_file(Path::new("encryption.key"))?;

    if key_path.exists() {
        let key = fs::read(&key_path)?;
        if key.len() == 32 {
            info!("Loaded encryption key from {}", key_path.display());
            return Ok(key);
        }
        warn!(
            "Invalid encryption key size ({} bytes), regenerating",
            key.len()
        );
    }

    // 新しい鍵を生成
    use rand::RngCore;
    let mut key = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    // 0600 で保存
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&key_path)?;
    file.write_all(&key)?;
    info!("Generated new encryption key at {}", key_path.display());

    Ok(key)
}

fn load_user_data(key: Option<&[u8]>) -> Arc<Mutex<UserData>> {
    match UserData::load_from_default_path(key) {
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

    let logpath = xdg::BaseDirectories::with_prefix("akaza")?
        .create_cache_directory("logs")?
        .join("ibus-akaza.log");
    println!("log file path: {}", logpath.to_string_lossy());

    // log file をファイルに書いていく。
    // ~/.cache/akaza/logs/ibus-akaza.log に書く。
    // https://superuser.com/questions/1293842/where-should-userspecific-application-log-files-be-stored-in-gnu-linux
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(arg.verbose.log_level_filter())
        .chain(std::io::stdout())
        .chain(fern::Output::writer(
            Box::new(FlushFile::new(logpath)?),
            "\n",
        ))
        .apply()?;

    info!("Starting ibus-akaza(rust version)");

    let encryption_key = match load_or_create_encryption_key() {
        Ok(key) => Some(key),
        Err(err) => {
            warn!(
                "Cannot load/create encryption key, user data will not be encrypted: {}",
                err
            );
            None
        }
    };

    unsafe {
        let sys_time = SystemTime::now();
        let user_data = load_user_data(encryption_key.as_deref());
        let config = Config::load()?;
        let akaza = BigramWordViterbiEngineBuilder::new(Config::load()?.engine)
            .user_data(user_data.clone())
            .build()?;
        let mut ac = AkazaContext::new(akaza, config);
        let new_sys_time = SystemTime::now();
        let difference = new_sys_time.duration_since(sys_time)?;
        info!(
            "Initialized ibus-akaza engine in {} milliseconds.",
            difference.as_millis()
        );

        // ユーザー辞書をバックグラウンドで保存するスレッド。
        thread::Builder::new()
            .name("user-data-save-thread".to_string())
            .spawn(move || {
                let interval = time::Duration::from_secs(3);

                // スレッド内で雑に例外投げるとスレッドとまっちゃうので丁寧めに処理する。
                loop {
                    if let Ok(mut data) = user_data.lock() {
                        if let Err(e) = data.write_user_files() {
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
            focus_in,
            property_activate,
        );

        ibus_akaza_init(arg.ibus);

        info!("Enter the ibus_main()");

        // run main loop
        ibus_main();

        warn!("Should not reach here.");
    }
    Ok(())
}

struct FlushFile {
    file: std::fs::File,
}

impl FlushFile {
    fn new(path: std::path::PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { file })
    }
}

impl Write for FlushFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = self.file.write(buf)?;
        self.file.flush()?;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
