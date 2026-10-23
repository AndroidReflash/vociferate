use crate::network::config::NetworkConfig;
use crate::network::connection::QuicConnection;
use crate::network::crypto::CryptoManager;
use crate::network::error::{NetworkError, Result};
use crate::protocol::{Capabilities, HandshakePayload};
use crate::discovery::MdnsService;
use quinn::{Endpoint, ServerConfig};
use std::net::SocketAddr;
use std::sync::{Arc};
use log::{error, debug, info};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::sync::Mutex as AsyncMutex;
use crate::uniffi_exports::{PeerInfo as UniffiPeerInfo};
use crate::network::get_cached_device_id;
use crate::uniffi_exports::callback::ConnectionEvent;
use crate::session::GlobalSession;
use crate::network::{PeerInfo as NetworkPeerInfo};
use crate::uniffi_exports::VoiceError;
use crate::session::SessionHandle;
use crate::session::SessionConfig;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub addr: SocketAddr,
    pub capabilities: Capabilities,
    pub session_id: u64,
    pub hostname: String,
}

type EventSender = mpsc::UnboundedSender<ConnectionEvent>;

pub struct ConnectionManager {
    config: NetworkConfig,
    crypto: Arc<CryptoManager>,
    endpoint: Option<Endpoint>,
    server_endpoint: Option<Endpoint>,
    pub connections: Arc<dashmap::DashMap<String, Arc<QuicConnection>>>,
    event_tx: EventSender,
    incoming_tx: Option<tokio::sync::mpsc::Sender<(String, Arc<QuicConnection>, PeerInfo)>>,
    _accept_handle: Option<JoinHandle<()>>,
    mdns: Option<MdnsService>,
    mdns_last_name: Option<String>,
}

impl ConnectionManager {
    pub fn new(config: NetworkConfig, event_tx: EventSender, device_name: &str, lan_ip: &str) -> Result<Self> {
        let crypto = Arc::new(CryptoManager::new()?);
        Ok(Self {
            config,
            crypto,
            endpoint: None,
            server_endpoint: None,
            connections: dashmap::DashMap::new().into(),
            event_tx,
            incoming_tx: None,
            _accept_handle: None,
            mdns: None,
            mdns_last_name: None,
        })
    }

    pub fn set_incoming_tx(&mut self, tx: tokio::sync::mpsc::Sender<(String, Arc<QuicConnection>, PeerInfo)>) {
        self.incoming_tx = Some(tx);
    }
    // принимающая сторона
    pub async fn start_listening(&mut self, addr: SocketAddr, self_arc: Arc<tokio::sync::Mutex<Self>>) -> Result<()> {
        let server_config = self.crypto.server_config(&self.config)?;
        let endpoint = Endpoint::server(server_config, addr)
            .map_err(|e| NetworkError::ConnectionFailed(format!("Server endpoint failed: {e}")))?;

        let event_tx = self.event_tx.clone();
        let crypto = self.crypto.clone();
        let config = self.config.clone();
        let incoming_tx = self.incoming_tx.clone();

        // Клонируем Arc мапы один раз из self, чтобы протащить его в фоновый таск
        let connections = self.connections.clone();

        // Клонируем self_arc для безопасного протаскивания в цикл
        let self_arc_clone = self_arc.clone();

        error!("start adress {:?}", addr);
        let endpoint_clone = endpoint.clone();

        let handle = tokio::spawn(async move {
            while let Some(incoming) = endpoint_clone.accept().await {
                let event_tx = event_tx.clone();
                let crypto = crypto.clone();
                let config = config.clone();
                let incoming_tx = incoming_tx.clone();

                // Передаем клон указателя внутрь обработчика входящего подключения
                let connections = connections.clone();
                let self_arc_clone = self_arc_clone.clone();

                tokio::spawn(async move {
                    match incoming.await {
                        Ok(conn) => {
                            info!("маладэц");
                            let peer_addr = conn.remote_address();

                            let mut quic = QuicConnection::new(conn, config);
                            let local_payload = HandshakePayload {
                                protocol_version: 1,
                                device_name: "VoiceTransfer".into(),
                                capabilities: Capabilities::default(),
                                session_id: rand::random(),
                                device_id: get_cached_device_id().to_string(),
                            };

                            match quic.perform_handshake_server(local_payload).await {
                                Ok(peer_payload) => {
                                    let peer_id = peer_payload.device_id.clone();
                                    let peer_info = PeerInfo {
                                        id: peer_id.clone(),
                                        name: peer_payload.device_name.clone(),
                                        addr: peer_addr,
                                        capabilities: peer_payload.capabilities.clone(),
                                        session_id: peer_payload.session_id,
                                        hostname: peer_payload.device_name.clone(),
                                    };

                                    let connection = Arc::new(quic);

                                    // Вставляем в общую мапу коннектов
                                    connections.insert(peer_id.clone(), connection.clone());

                                    // 1. Регистрируем коннект в GlobalSession
                                    GlobalSession::global().set_connection(peer_id.clone(), connection.clone());

                                    // 2. Запускаем глобальную сессию / добавляем пир в синглтон
                                    let network_peer_info = NetworkPeerInfo {
                                        id: peer_id.clone(),
                                        name: peer_payload.device_name.clone(),
                                        addr: peer_addr,
                                        capabilities: peer_payload.capabilities.clone(),
                                        session_id: peer_payload.session_id,
                                        hostname: peer_payload.device_name.clone(),
                                    };

                                    if let Err(e) = GlobalSession::global().start(vec![network_peer_info]).await {
                                        log::error!("Failed to start global session for peer {}: {:?}", peer_id, e);
                                    }

                                    let uniffi_peer = crate::uniffi_exports::PeerInfo {
                                        id: peer_id.clone(),
                                        name: peer_payload.device_name.clone(),
                                        ip: peer_addr.ip().to_string(),
                                        port: peer_addr.port(),
                                        hostname: peer_payload.device_name.clone(),
                                    };

                                    let uniffi_peer_clone = uniffi_peer.clone();

                                    let _ = crate::uniffi_exports::callback::CONNECTION_CHANNEL.0.send(
                                         crate::uniffi_exports::callback::ConnectionEvent::Connected(
                                             uniffi_peer
                                         )
                                    );

                                    if let Some(tx) = &incoming_tx {
                                        let _ = tx.send((peer_id.clone(), connection.clone(), peer_info.clone())).await;
                                    }

                                    let conn_clone = connection.clone();
                                    let peer_id_clone = peer_id.clone();
                                    let connections_clone = connections.clone();

                                    // Фоновая таска на отслеживание разрыва соединения
                                    tokio::spawn(async move {
                                        let _closed_reason = conn_clone.connection.closed().await;

                                        connections_clone.remove(&peer_id_clone);

                                        // Корректно удаляем пир из GlobalSession при отключении
                                        GlobalSession::global().remove_peer(&peer_id_clone).await;

                                        let _ = crate::uniffi_exports::callback::CONNECTION_CHANNEL.0.send(
                                             crate::uniffi_exports::callback::ConnectionEvent::Disconnected(
                                                  uniffi_peer_clone.id
                                             )
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

        self._accept_handle = Some(handle);
        self.server_endpoint = Some(endpoint);
        Ok(())
    }

    pub async fn connect(&mut self, addr: SocketAddr, server_name: &str) -> Result<PeerInfo> {
        if self.endpoint.is_none() {
            let client_config = crate::network::crypto::make_client_config(self.crypto.client_roots())?;
            let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
                .map_err(|e| NetworkError::ConnectionFailed(format!("Client endpoint failed: {e}")))?;
            endpoint.set_default_client_config(client_config);
            self.endpoint = Some(endpoint);
        }
        let endpoint = self.endpoint.as_ref().unwrap();
        let connecting = endpoint
            .connect(addr, server_name)
            .map_err(|e| NetworkError::ConnectionFailed(format!("Connect failed: {e}")))?;
        let conn = connecting
            .await
            .map_err(|e| NetworkError::ConnectionFailed(format!("Connection failed: {e}")))?;

        let mut quic = QuicConnection::new(conn, self.config.clone());
        let local_payload = HandshakePayload {
            protocol_version: 1,
            device_name: "VoiceTransfer".into(),
            capabilities: Capabilities::default(),
            session_id: rand::random(),
            device_id: get_cached_device_id().to_string(),
        };
        let peer_payload = quic.perform_handshake(local_payload).await?;
        let peer_id = peer_payload.device_id.clone();
        let peer_info = PeerInfo {
            id: peer_id.clone(),
            name: peer_payload.device_name.clone(),
            addr,
            capabilities: peer_payload.capabilities,
            session_id: peer_payload.session_id,
            hostname: peer_payload.device_name.clone(),
        };

        let connection = Arc::new(quic);
        self.connections.insert(peer_id.clone(), connection.clone());

        let uniffi_peer = crate::uniffi_exports::PeerInfo {
            id: peer_id.clone(),
            name: peer_payload.device_name.clone(),
            ip: addr.ip().to_string(),
            port: addr.port(),
            hostname: peer_payload.device_name.clone(),
        };

        // Шлем событие коннекта в канал через Sender
        let uniffi_peer_clone = uniffi_peer.clone();

        let _ = crate::uniffi_exports::callback::CONNECTION_CHANNEL.0.send(
                crate::uniffi_exports::callback::ConnectionEvent::Connected(
                     uniffi_peer
                )
        );

        let conn_clone = connection.clone();
        let peer_id_clone = peer_id.clone();
        let connections_clone = self.connections.clone();
        let event_tx_clone = self.event_tx.clone();

        tokio::spawn(async move {
            // Ждем пока соединение сдохнет
            let _closed_reason = conn_clone.connection.closed().await;

            // Чистим локальную мапу
            connections_clone.remove(&peer_id_clone);
            info!("клиент отвалился: {}", peer_id_clone);

            // Шлем Disconnected в ту же трубу для Kotlin
            let _ = crate::uniffi_exports::callback::CONNECTION_CHANNEL.0.send(
                    crate::uniffi_exports::callback::ConnectionEvent::Disconnected(
                         uniffi_peer_clone.id
                    )
            );
        });

        Ok(peer_info)
    }

    pub fn get_connection(&self, peer_id: &str) -> Option<Arc<QuicConnection>> {
        self.connections.get(peer_id).map(|v| v.clone())
    }

   pub async fn disconnect(&mut self, peer_id: &str) -> Result<()> {
       if let Some((_, conn)) = self.connections.remove(peer_id) {
           info!("disconnect: killing peer {}", peer_id);

           conn.connection.close(1u32.into(), b"forced_disconnect");

           let close_fut = conn.connection.closed();

           info!("убийство отработал");
       }
       Ok(())
   }

    pub fn set_advertising_name(&mut self, name: &str) {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            self.mdns_last_name = Some(trimmed.to_string());
        }
    }

    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.server_endpoint.as_ref().and_then(|e| e.local_addr().ok())
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        if let Some(handle) = self._accept_handle.take() {
            handle.abort();
        }
    }
}