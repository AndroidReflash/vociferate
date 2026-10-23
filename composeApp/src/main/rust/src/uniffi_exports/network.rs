use crate::network::{
    ConnectionManager, NetworkConfig, NetworkError, PeerInfo as NetworkPeerInfo,
    QuicConnection,
};
use crate::protocol::Capabilities;
use crate::session::{GlobalSession, SessionConfig, SessionEvent};
use crate::uniffi_exports::{VoiceError, PeerInfo, SessionHandle,
    RUNTIME, SHARED_CONNECTIONS};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::uniffi_exports::SETTING_KEY_DEVICE_NAME;
use tokio::sync::Mutex;

#[uniffi::export]
pub async fn start_server(addr: String) -> Result<(), VoiceError> {
    RUNTIME.spawn(async move {
        let socket_addr: SocketAddr = addr
            .parse()
            .map_err(|e: std::net::AddrParseError| VoiceError::InvalidAddress(e.to_string()))?;

        let db = crate::uniffi_exports::get_db()?;
        let device_name = db
            .get(SETTING_KEY_DEVICE_NAME.to_string())
            .map_err(|e| VoiceError::Other(e.to_string()))?
            .unwrap_or_else(|| "VoiceTransfer".to_string());
        let lan_ip = crate::uniffi_exports::get_lan_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let config = NetworkConfig::default();
        let (conn_tx, _conn_rx) = mpsc::unbounded_channel();

        let manager = ConnectionManager::new(config, conn_tx, &device_name, &lan_ip)
            .map_err(|e| VoiceError::Other(e.to_string()))?;
        let shared_cm = Arc::new(Mutex::new(manager));

        shared_cm
            .lock()
            .await
            .start_listening(socket_addr, shared_cm.clone())
            .await
            .map_err(|e| VoiceError::Network(e.to_string()))?;

        {
            let mut shared = SHARED_CONNECTIONS.lock().unwrap();
            *shared = Some(shared_cm);
        }

        if let Err(e) = GlobalSession::global().start(vec![]).await {
            log::error!("Не удалось запустить GlobalSession: {}", e);
        }

        Ok(())
    })
    .await
    .map_err(|e| VoiceError::Other(e.to_string()))?
}

 // отдающая сторона
#[uniffi::export]
pub fn connect_to_peer(peer: PeerInfo) -> Result<SessionHandle, VoiceError> {
    RUNTIME.block_on(async move {
        let _ = rustls::crypto::ring::default_provider().install_default();
        log::info!("подключаемся и принимаем звук от {:?}", peer);

        let connections = {
            let shared = SHARED_CONNECTIONS.lock().unwrap();
            match shared.as_ref() {
                Some(cm) => cm.clone(),
                None => {
                    return Err(VoiceError::Other(
                        "Discovery not started. Call start_discovery() first.".to_string(),
                    ))
                }
            }
        };

        let handle = SessionHandle {
            inner: crate::uniffi_exports::NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        };

        let _network_peer = connect_peer(connections, &peer).await?;

        Ok(handle)
    })
}

pub async fn connect_peer(
    connections: Arc<tokio::sync::Mutex<ConnectionManager>>,
    peer: &PeerInfo,
) -> Result<NetworkPeerInfo, VoiceError> {
    let addr = peer.socket_addr()?;
    let server_name = if !peer.hostname.is_empty() {
        peer.name.clone()
    } else {
        peer.ip.clone()
    };

    let network_peer = {
        let mut cm = connections.lock().await;
        cm.connect(addr, &server_name).await?
    };

    let connection = {
        let cm = connections.lock().await;
        cm.get_connection(&network_peer.id)
            .ok_or_else(|| VoiceError::Network(format!("no connection for {}", network_peer.id)))?
    };

    GlobalSession::global().set_connection(network_peer.id.clone(), connection);

    GlobalSession::global()
        .start(vec![network_peer.clone()])
        .await
        .map_err(|e| VoiceError::Session(e))?;

    Ok(network_peer)
}

#[uniffi::export]
pub async fn disconnect_peer(device_id: String) -> Result<(), VoiceError> {
    let cm = {
        let shared = SHARED_CONNECTIONS.lock().unwrap();
        shared.as_ref().cloned()
            .ok_or(VoiceError::Network("No active connection manager".into()))?
    };
    let mut cm = cm.lock().await;
    cm.disconnect(&device_id).await.map_err(|e| VoiceError::Network(e.to_string()))
}