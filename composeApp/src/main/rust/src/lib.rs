mod protocol;
mod database;
mod codec;
mod audio;
mod jitter;
mod discovery;
mod network;
mod session;
mod uniffi_exports;
mod testkit;
mod android_context;

use uniffi;

uniffi::setup_scaffolding!();

#[no_mangle]
pub extern "C" fn JNI_OnLoad(_: *mut std::ffi::c_void, _: *mut std::ffi::c_void) -> i32 {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("RUSTRUST"),
        );
    }
    0x00010006
}

#[uniffi::export]
pub fn init_rust_logging() {
    use env_logger::{Builder, Env};
    use log::LevelFilter;

    let env = Env::default().default_filter_or("debug");
    let _ = Builder::from_env(env)
        .filter_level(LevelFilter::Debug)
        .target(env_logger::Target::Stdout)
        .try_init();

    log::info!("VoiceTransfer Rust logger initialized!");
}