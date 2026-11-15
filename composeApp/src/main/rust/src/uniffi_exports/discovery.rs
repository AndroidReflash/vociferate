use std::net::SocketAddr;

use tokio::sync::mpsc;

use crate::discovery::{GlobalDiscoveryReceiver, GlobalDiscoverySender};
use crate::network::{ConnectionManager, NetworkConfig};
use crate::protocol::Capabilities;
use crate::uniffi_exports::error::VoiceError;
use crate::uniffi_exports::runtime::RUNTIME;
use crate::uniffi_exports::state::{get_db, get_lan_ip, INCOMING_CHANNEL_CAPACITY, SETTING_KEY_DEVICE_NAME};

#[uniffi::export(async_runtime = "tokio")]
pub async fn start_discovery() -> Result<(), VoiceError> {
    RUNTIME.spawn(async move {
        let db = get_db()?;
        let device_name = db
            .get(SETTING_KEY_DEVICE_NAME.to_string())
            .map_err(|e| VoiceError::Other(e.to_string()))?
            .unwrap_or_else(|| "VoiceTransfer".to_string());
        let lan_ip = get_lan_ip()
            .ok_or_else(|| VoiceError::Network("Failed to determine LAN IP".to_string()))?;

        let config = NetworkConfig::default();
        let (conn_tx, _conn_rx) = mpsc::unbounded_channel();
        let manager = ConnectionManager::init(config, conn_tx)
            .map_err(|e| VoiceError::Other(e.to_string()))?;

        let (incoming_tx, mut _incoming_rx) = mpsc::channel(INCOMING_CHANNEL_CAPACITY);
        manager.set_incoming_tx(incoming_tx).await;

        let actual_port = manager.local_addr().await.map(|a| a.port()).unwrap_or(0);

        let local_addr = SocketAddr::new(lan_ip, actual_port);
        let capabilities = Capabilities::default();
        let device_id = crate::network::config::get_cached_device_id().to_string();
        let session_id = rand::random();

        GlobalDiscoverySender::global().start(
            device_name.clone(),
            device_id,
            local_addr,
            capabilities,
            session_id,
        );

        GlobalDiscoveryReceiver::global().start(device_name, lan_ip.to_string());

        log::info!("Discovery sender and receiver singletons started on {}:{}", lan_ip, actual_port);
        Ok(())
    })
    .await
    .map_err(|e| VoiceError::Other(e.to_string()))?
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn stop_discovery() -> Result<(), VoiceError> {
    GlobalDiscoverySender::global().stop();
    GlobalDiscoveryReceiver::global().stop();

    if let Ok(manager) = ConnectionManager::global() {
        manager.shutdown().await;
    }
    log::info!("Discovery singletons stopped");
    Ok(())
}


