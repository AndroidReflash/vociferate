//! Discovery UniFFI exports

use crate::discovery::{DiscoveryError, MdnsService};
use crate::network::{NetworkConfig, ConnectionManager};
use crate::protocol::Capabilities;
use crate::session::{SessionConfig, SessionEvent, SessionHandle, SessionStats};
use crate::uniffi_exports::{VoiceError, PeerInfo,
    RUNTIME, NEXT_HANDLE, SHARED_CONNECTIONS,
    INCOMING_CHANNEL_CAPACITY, SETTING_KEY_DEVICE_NAME};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::Duration;
use crate::discovery::{ GlobalDiscoverySender, GlobalDiscoveryReceiver };



#[uniffi::export]
pub async fn start_discovery() -> Result<(), VoiceError> {
    RUNTIME.spawn(async move {
        let db = crate::uniffi_exports::get_db()?;
        let device_name = db
            .get(SETTING_KEY_DEVICE_NAME.to_string())
            .map_err(|e| VoiceError::Other(e.to_string()))?
            .unwrap_or_else(|| "VoiceTransfer".to_string());
        let lan_ip = crate::uniffi_exports::get_lan_ip()
            .ok_or_else(|| VoiceError::Network("Failed to determine LAN IP".to_string()))?;

        let config = NetworkConfig::default();
        let (conn_tx, _conn_rx) = mpsc::unbounded_channel();
        let mut connections =
            ConnectionManager::new(config, conn_tx, &device_name, &lan_ip.to_string())?;

        let (incoming_tx, mut incoming_rx) = mpsc::channel(INCOMING_CHANNEL_CAPACITY);
        connections.set_incoming_tx(incoming_tx);

        let actual_port = connections.local_addr().map(|a| a.port()).unwrap_or(0);

        let shared_cm = Arc::new(tokio::sync::Mutex::new(connections));
        {
            *SHARED_CONNECTIONS.lock().unwrap() = Some(shared_cm.clone());
        }

        let local_addr = SocketAddr::new(lan_ip, actual_port);
        let capabilities = Capabilities::default();
        let device_id = crate::network::config::get_cached_device_id().to_string();
        let session_id = rand::random();

        // запускаем синглтон отправки
        GlobalDiscoverySender::global().start(
            device_name.clone(),
            device_id,
            local_addr,
            capabilities,
            session_id,
        );

        // запускаем синглтон приема
        GlobalDiscoveryReceiver::global().start(
            device_name.clone(),
            lan_ip.to_string(),
        );

        log::info!("Discovery sender and receiver singletons started on {}:{}", lan_ip, actual_port);
        Ok(())
    })
    .await
    .map_err(|e| VoiceError::Other(e.to_string()))?
}

#[uniffi::export]
pub fn stop_discovery() -> Result<(), VoiceError> {
    GlobalDiscoverySender::global().stop();

    GlobalDiscoveryReceiver::global().stop();

    {
        let mut shared = SHARED_CONNECTIONS.lock().unwrap();
        *shared = None;
    }
    log::info!("Discovery singletons stopped");
    Ok(())
}


