use std::sync::LazyLock;
use tokio::sync::{mpsc, Mutex};

use crate::uniffi_exports::types::PeerInfo;

#[derive(uniffi::Enum, Clone, Debug)]
pub enum ConnectionEvent {
    Connected(PeerInfo),
    Disconnected(String),
    AudioData(Vec<u8>),
    Error(String),
}

type ConnectionChannel = (
    mpsc::UnboundedSender<ConnectionEvent>,
    Mutex<mpsc::UnboundedReceiver<ConnectionEvent>>
);

pub static CONNECTION_CHANNEL: LazyLock<ConnectionChannel> = LazyLock::new(|| {
    let (tx, rx) = mpsc::unbounded_channel();
    (tx, Mutex::new(rx))
});

#[uniffi::export]
pub fn next_connection_event() -> Option<ConnectionEvent> {
    crate::uniffi_exports::runtime::RUNTIME.block_on(async {
        CONNECTION_CHANNEL.1.lock().await.recv().await
    })
}