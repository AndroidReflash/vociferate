use crate::network::config::NetworkConfig;
use crate::network::connection::QuicConnection;
use crate::network::crypto::CryptoManager;
use crate::network::error::{NetworkError, Result};
use crate::protocol::{Capabilities, HandshakePayload};
use quinn::Endpoint;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use log::{error, info};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::sync::{Mutex as AsyncMutex, OnceCell};
use crate::network::get_cached_device_id;
use crate::uniffi_exports::{ConnectionEvent, CONNECTION_CHANNEL, PeerInfo as UniffiPeerInfo};
use crate::session::GlobalSession;
use crate::network::{PeerInfo as NetworkPeerInfo};

static CONNECTION_MANAGER: OnceLock<ConnectionManager> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub addr: SocketAddr,
    pub hostname: String,
    pub is_active_micro: bool,
}

type EventSender = mpsc::UnboundedSender<ConnectionEvent>;

pub struct ConnectionManager {
    config: NetworkConfig,
    crypto: Arc<CryptoManager>,
    client_endpoint: OnceCell<Endpoint>,
    server_endpoint: AsyncMutex<Option<Endpoint>>,
    pub connections: Arc<dashmap::DashMap<String, Arc<QuicConnection>>>,
    event_tx: EventSender,
    incoming_tx: AsyncMutex<Option<mpsc::Sender<(String, Arc<QuicConnection>, PeerInfo)>>>,
    accept_handle: AsyncMutex<Option<JoinHandle<()>>>,
    inflight_connects: dashmap::DashMap<String, Arc<AsyncMutex<()>>>,
}

impl ConnectionManager {
    fn new(config: NetworkConfig, event_tx: EventSender) -> Result<Self> {
        let crypto = Arc::new(CryptoManager::new()?);
        Ok(Self {
            config,
            crypto,
            client_endpoint: OnceCell::new(),
            server_endpoint: AsyncMutex::new(None),
            connections: Arc::new(dashmap::DashMap::new()),
            event_tx,
            incoming_tx: AsyncMutex::new(None),
            accept_handle: AsyncMutex::new(None),
            inflight_connects: dashmap::DashMap::new(),
        })
    }

    pub fn global() -> Result<&'static Self> {
        CONNECTION_MANAGER
            .get()
            .ok_or_else(|| NetworkError::InvalidState("Connection manager not initialized".to_string()))
    }

    pub fn init(config: NetworkConfig, event_tx: EventSender) -> Result<&'static Self> {
        let manager = Self::new(config, event_tx)?;
        let _ = CONNECTION_MANAGER.set(manager);
        Self::global()
    }

    pub async fn set_incoming_tx(&self, tx: mpsc::Sender<(String, Arc<QuicConnection>, PeerInfo)>) {
        *self.incoming_tx.lock().await = Some(tx);
    }

    async fn get_or_init_client_endpoint(&self) -> Result<Endpoint> {
        self.client_endpoint
            .get_or_try_init(|| async {
                let client_config = crate::network::crypto::make_client_config(self.crypto.client_roots())?;
                let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
                    .map_err(|e| NetworkError::ConnectionFailed(format!("Client endpoint failed: {e}")))?;
                endpoint.set_default_client_config(client_config);
                Ok(endpoint)
            })
            .await
            .cloned()
    }

    // принимающая сторона
    pub async fn start_listening(&self, addr: SocketAddr) -> Result<()> {
        let server_config = self.crypto.server_config(&self.config)?;
        let endpoint = Endpoint::server(server_config, addr)
            .map_err(|e| NetworkError::ConnectionFailed(format!("Server endpoint failed: {e}")))?;

        let event_tx = self.event_tx.clone();
        let config = self.config.clone();
        let incoming_tx = self.incoming_tx.lock().await.clone();
        let connections = self.connections.clone();

        error!("start adress {:?}", addr);
        let endpoint_clone = endpoint.clone();

        let handle = tokio::spawn(async move {
            while let Some(incoming) = endpoint_clone.accept().await {
                let event_tx = event_tx.clone();
                let config = config.clone();
                let incoming_tx = incoming_tx.clone();
                let connections = connections.clone();

                tokio::spawn(async move {
                    match incoming.await {
                        Ok(conn) => {
                            info!("мы друг друга видим");
                            let peer_addr = conn.remote_address();

                            let mut quic = QuicConnection::new(conn, config);
                            let (current_name, current_mic_status) = {
                                if let Some(payload) = crate::discovery::GlobalDiscoverySender::global().payload.lock().unwrap().as_ref() {
                                    (payload.device_name.clone(), payload.is_active_micro)
                                } else {
                                    ("VoiceTransfer".to_string(), false)
                                }
                            };

                            let local_payload = HandshakePayload {
                                protocol_version: 1,
                                device_name: current_name,
                                capabilities: Capabilities::default(),
                                session_id: rand::random(),
                                device_id: get_cached_device_id().to_string(),
                                is_active_micro: current_mic_status,
                            };

                            match quic.perform_handshake_server(local_payload).await {
                                Ok(peer_payload) => {
                                    let peer_id = peer_payload.device_id.clone();
                                    let peer_info = PeerInfo {
                                        id: peer_id.clone(),
                                        name: peer_payload.device_name.clone(),
                                        addr: peer_addr,
                                        hostname: peer_payload.device_name.clone(),
                                        is_active_micro: peer_payload.is_active_micro.clone(),
                                    };

                                    let connection = Arc::new(quic);
                                    connections.insert(peer_id.clone(), connection.clone());

                                    GlobalSession::global().set_connection(peer_id.clone(), connection.clone());

                                    let network_peer_info = NetworkPeerInfo {
                                        id: peer_id.clone(),
                                        name: peer_payload.device_name.clone(),
                                        addr: peer_addr,
                                        hostname: peer_payload.device_name.clone(),
                                        is_active_micro: peer_payload.is_active_micro.clone(),
                                    };

                                    if let Err(e) = GlobalSession::global().start(vec![network_peer_info]).await {
                                        log::error!("Failed to start global session for peer {}: {:?}", peer_id, e);
                                    }

                                    let uniffi_peer = UniffiPeerInfo {
                                        id: peer_id.clone(),
                                        name: peer_payload.device_name.clone(),
                                        ip: peer_addr.ip().to_string(),
                                        port: peer_addr.port(),
                                        hostname: peer_payload.device_name.clone(),
                                        is_active_micro: peer_payload.is_active_micro.clone(),
                                    };

                                    let _ = CONNECTION_CHANNEL.0.send(ConnectionEvent::Connected(
                                        uniffi_peer,
                                    ));

                                    if let Some(tx) = &incoming_tx {
                                        let _ = tx.send((peer_id.clone(), connection.clone(), peer_info.clone())).await;
                                    }

                                    let conn_clone = connection.clone();
                                    let peer_id_clone = peer_id.clone();
                                    let connections_clone = connections.clone();

                                    tokio::spawn(async move {
                                        let _closed_reason = conn_clone.connection.closed().await;
                                        connections_clone.remove(&peer_id_clone);
                                        GlobalSession::global().remove_peer(&peer_id_clone).await;
                                        let _ = CONNECTION_CHANNEL.0.send(
                                            ConnectionEvent::Disconnected(peer_id_clone),
                                        );
                                    });
                                }
                                Err(e) => {
                                    let _ = event_tx.send(ConnectionEvent::Error(format!(
                                        "Handshake failed for {}: {}",
                                        peer_addr, e
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = event_tx.send(ConnectionEvent::Error(format!(
                                "Incoming connection failed: {}",
                                e
                            )));
                        }
                    }
                });
            }
        });

        *self.accept_handle.lock().await = Some(handle);
        *self.server_endpoint.lock().await = Some(endpoint);
        Ok(())
    }

    pub async fn connect(&self, addr: SocketAddr, server_name: &str) -> Result<(PeerInfo, Arc<QuicConnection>)> {
        let addr_key = addr.to_string();
        let inflight = self
            .inflight_connects
            .entry(addr_key.clone())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone();
        let _guard = inflight.lock().await;

        let result = async {
            for entry in self.connections.iter() {
                if entry.value().connection.remote_address() == addr {
                    let id = entry.key().clone();
                    let connection = entry.value().clone();
                    let peer_info = PeerInfo {
                        id,
                        name: server_name.to_string(),
                        addr,
                        hostname: server_name.to_string(),
                        is_active_micro: false,
                    };
                    return Ok((peer_info, connection));
                }
            }

            let endpoint = self.get_or_init_client_endpoint().await?;
            let config = self.config.clone();
            let connections = self.connections.clone();
            let event_tx = self.event_tx.clone();

            let connecting = endpoint
                .connect(addr, server_name)
                .map_err(|e| NetworkError::ConnectionFailed(format!("Connect failed: {e}")))?;
            let conn = connecting
                .await
                .map_err(|e| NetworkError::ConnectionFailed(format!("Connection failed: {e}")))?;

            let mut quic = QuicConnection::new(conn, config);
            let (current_name, current_mic_status) = {
                if let Some(payload) = crate::discovery::GlobalDiscoverySender::global().payload.lock().unwrap().as_ref() {
                    (payload.device_name.clone(), payload.is_active_micro)
                } else {
                    ("VoiceTransfer".to_string(), false)
                }
            };

            let local_payload = HandshakePayload {
                protocol_version: 1,
                device_name: current_name,
                capabilities: Capabilities::default(),
                session_id: rand::random(),
                device_id: get_cached_device_id().to_string(),
                is_active_micro: current_mic_status,
            };
            let peer_payload = quic.perform_handshake(local_payload).await?;
            let peer_id = peer_payload.device_id.clone();
            let peer_info = PeerInfo {
                id: peer_id.clone(),
                name: peer_payload.device_name.clone(),
                addr,
                hostname: peer_payload.device_name.clone(),
                is_active_micro: peer_payload.is_active_micro.clone(),
            };

            let connection = Arc::new(quic);
            connections.insert(peer_id.clone(), connection.clone());

            let uniffi_peer = UniffiPeerInfo {
                id: peer_id.clone(),
                name: peer_payload.device_name.clone(),
                ip: addr.ip().to_string(),
                port: addr.port(),
                hostname: peer_payload.device_name.clone(),
                is_active_micro: peer_payload.is_active_micro.clone(),
            };

            let uniffi_peer_clone = uniffi_peer.clone();
            let _ = CONNECTION_CHANNEL.0.send(ConnectionEvent::Connected(uniffi_peer));

            let conn_clone = connection.clone();
            let peer_id_clone = peer_id.clone();
            let connections_clone = connections.clone();
            let event_tx_clone = event_tx.clone();

            tokio::spawn(async move {
                let _closed_reason = conn_clone.connection.closed().await;
                connections_clone.remove(&peer_id_clone);
                GlobalSession::global().remove_peer(&peer_id_clone).await;
                info!("клиент отвалился: {}", peer_id_clone);
                let _ = event_tx_clone.send(ConnectionEvent::Disconnected(
                    uniffi_peer_clone.id.clone(),
                ));
                let _ = CONNECTION_CHANNEL.0.send(ConnectionEvent::Disconnected(
                    peer_id_clone,
                ));
            });

            Ok((peer_info, connection))
        }
        .await;

        self.inflight_connects.remove(&addr_key);
        result
    }

    pub async fn disconnect(&self, peer_id: &str) -> Result<()> {
        if let Some((_, conn)) = self.connections.remove(peer_id) {
            info!("disconnect: killing peer {}", peer_id);
            conn.connection.close(1u32.into(), b"forced_disconnect");
            let _ = conn.connection.closed().await;
            info!("убийство отработал");
        }
        Ok(())
    }

    pub async fn local_addr(&self) -> Option<SocketAddr> {
        self.server_endpoint
            .lock()
            .await
            .as_ref()
            .and_then(|e| e.local_addr().ok())
    }

    pub async fn shutdown(&self) {
        if let Some(handle) = self.accept_handle.lock().await.take() {
            handle.abort();
        }
        if let Some(endpoint) = self.server_endpoint.lock().await.take() {
            endpoint.close(0u32.into(), b"shutdown");
        }
        if let Some(endpoint) = self.client_endpoint.get() {
            endpoint.close(0u32.into(), b"shutdown");
        }
        let ids: Vec<String> = self.connections.iter().map(|e| e.key().clone()).collect();
        for (_, conn) in self.connections.iter().map(|e| (e.key().clone(), e.value().clone())).collect::<Vec<_>>() {
            conn.connection.close(0u32.into(), b"shutdown");
        }
        self.connections.clear();
        for id in ids {
            GlobalSession::global().remove_peer(&id).await;
        }
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        if let Ok(mut handle) = self.accept_handle.try_lock() {
            if let Some(handle) = handle.take() {
                handle.abort();
            }
        }
    }
}