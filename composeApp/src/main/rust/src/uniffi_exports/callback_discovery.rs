use tokio::sync::{mpsc, Mutex};
use std::sync::LazyLock;

#[derive(uniffi::Enum, Clone, Debug)]
pub enum DiscoveryEvent {
    PeersAdded(Vec<crate::uniffi_exports::PeerInfo>),
    PeersRemoved(Vec<String>),
}

type DiscoveryChannel = (
    mpsc::UnboundedSender<DiscoveryEvent>,
    Mutex<mpsc::UnboundedReceiver<DiscoveryEvent>>
);

pub static DISCOVERY_CHANNEL: LazyLock<DiscoveryChannel> = LazyLock::new(|| {
    let (tx, rx) = mpsc::unbounded_channel();
    (tx, Mutex::new(rx))
});

#[uniffi::export]
pub fn next_discovery_event() -> Option<DiscoveryEvent> {
    crate::uniffi_exports::RUNTIME.block_on(async {
        DISCOVERY_CHANNEL.1.lock().await.recv().await
    })
}