use std::net::SocketAddr;
use std::sync::OnceLock;

use tokio::sync::mpsc;

use crate::network::{ConnectionManager, NetworkConfig, PeerInfo as NetworkPeerInfo};
use crate::session::GlobalSession;
use crate::uniffi_exports::error::VoiceError;
use crate::uniffi_exports::runtime::{NEXT_HANDLE, RUNTIME};
use crate::uniffi_exports::state::{get_db, get_lan_ip, SETTING_KEY_DEVICE_NAME};
use crate::uniffi_exports::types::{PeerInfo, SessionHandle};

static CRYPTO_INIT: OnceLock<()> = OnceLock::new();

fn ensure_crypto_provider() {
    CRYPTO_INIT.get_or_init(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn start_server(addr: String) -> Result<(), VoiceError> {
    RUNTIME.spawn(async move {
        let socket_addr: SocketAddr = addr
            .parse()
            .map_err(|e: std::net::AddrParseError| VoiceError::InvalidAddress(e.to_string()))?;

        let db = get_db()?;
        let _device_name = db
            .get(SETTING_KEY_DEVICE_NAME.to_string())
            .map_err(|e| VoiceError::Other(e.to_string()))?
            .unwrap_or_else(|| "VoiceTransfer".to_string());
        let _lan_ip = get_lan_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let config = NetworkConfig::default();
        let (conn_tx, _conn_rx) = mpsc::unbounded_channel();

        let manager = ConnectionManager::init(config, conn_tx)
            .map_err(|e| VoiceError::Other(e.to_string()))?;

        manager
            .start_listening(socket_addr)
            .await
            .map_err(|e| VoiceError::Network(e.to_string()))?;

        if let Err(e) = GlobalSession::global().start(vec![]).await {
            log::error!("Не удалось запустить GlobalSession: {}", e);
        }

        Ok(())
    })
    .await
    .map_err(|e| VoiceError::Other(e.to_string()))?
}

// отдающая сторона
#[uniffi::export(async_runtime = "tokio")]
pub async fn connect_to_peer(peer: PeerInfo) -> Result<SessionHandle, VoiceError> {
    ensure_crypto_provider();
    log::info!("подключаемся и принимаем звук от {:?}", peer);

    let handle = SessionHandle {
        inner: NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    };

    let _network_peer = connect_peer(&peer).await?;

    Ok(handle)
}

pub async fn connect_peer(peer: &PeerInfo) -> Result<NetworkPeerInfo, VoiceError> {
    let addr = peer.socket_addr()?;
    let server_name = if !peer.hostname.is_empty() {
        peer.name.clone()
    } else {
        peer.ip.clone()
    };

    let manager = ConnectionManager::global().map_err(|e| VoiceError::Network(e.to_string()))?;
    let (network_peer, connection) = manager.connect(addr, &server_name).await?;

    GlobalSession::global().set_connection(network_peer.id.clone(), connection);
    GlobalSession::global()
        .start(vec![network_peer.clone()])
        .await
        .map_err(VoiceError::Session)?;

    Ok(network_peer)
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn disconnect_peer(device_id: String) -> Result<(), VoiceError> {
    let manager = ConnectionManager::global().map_err(|e| VoiceError::Network(e.to_string()))?;
    manager
        .disconnect(&device_id)
        .await
        .map_err(|e| VoiceError::Network(e.to_string()))?;
    GlobalSession::global().remove_peer(&device_id).await;
    Ok(())
}