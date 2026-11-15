use std::sync::atomic::AtomicU64;
use std::sync::LazyLock;

pub static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for VoiceTransfer core")
});

pub static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
