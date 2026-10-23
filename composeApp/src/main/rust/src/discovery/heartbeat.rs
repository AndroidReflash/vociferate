use crate::protocol::Capabilities;
use crate::network::PeerInfo as NetworkPeerInfo;
use crate::uniffi_exports::{PeerInfo as UniffiPeerInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::net::UdpSocket;
use log::{debug, info, warn, error};
use std::sync::OnceLock;

const HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);
const PEER_TTL: std::time::Duration = std::time::Duration::from_secs(10);
const BROADCAST_ADDR: &str = "255.255.255.255:5354";
const CLEANUP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

const RECEIVER_BIND_ADDR: &str = "0.0.0.0:5354";
const SENDER_BIND_ADDR: &str = "0.0.0.0:0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub device_name: String,
    pub device_id: String,
    pub ip: String,
    pub port: u16,
    pub capabilities: Capabilities,
    pub session_id: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub info: NetworkPeerInfo,
    pub last_seen: Instant,
}


pub struct GlobalDiscoverySender {
    socket: Arc<UdpSocket>,
    sender_handle: Mutex<Option<JoinHandle<()>>>,
    shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl GlobalDiscoverySender {
    pub fn global() -> &'static Self {
         static INSTANCE: OnceLock<GlobalDiscoverySender> = OnceLock::new();
         INSTANCE.get_or_init(|| {
            let std_socket = std::net::UdpSocket::bind(SENDER_BIND_ADDR).expect("Failed to bind sender socket");
            std_socket.set_broadcast(true).expect("Failed set broadcast");

            let socket = UdpSocket::from_std(std_socket).expect("Failed to convert std socket");
            Self {
                socket: socket.into(),
                sender_handle: Mutex::new(None),
                shutdown_tx: Mutex::new(None),
            }
        })
    }

    pub fn start(&self, local_name: String, device_id: String, local_addr: SocketAddr, capabilities: Capabilities, session_id: u64) {
        let mut handle_lock = self.sender_handle.lock().unwrap();
        if handle_lock.is_some() {
            return;
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        let socket = self.socket.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            let bcast_addr: SocketAddr = BROADCAST_ADDR.parse().unwrap();

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let payload = HeartbeatPayload {
                            device_name: local_name.clone(),
                            device_id: device_id.clone(),
                            ip: local_addr.ip().to_string(),
                            port: local_addr.port(),
                            capabilities: capabilities.clone(),
                            session_id,
                            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        };
                        match serde_json::to_vec(&payload) {
                            Ok(data) => {
                                if let Err(e) = socket.send_to(&data, bcast_addr).await {
                                    warn!("Failed to send broadcast heartbeat: {}", e);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to serialize heartbeat: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        *handle_lock = Some(handle);
    }

    pub fn stop(&self) {
        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.try_send(());
        }
        if let Some(h) = self.sender_handle.lock().unwrap().take() {
            h.abort();
        }
    }
}


pub struct GlobalDiscoveryReceiver {
    socket: Arc<UdpSocket>,
    peers: Arc<Mutex<HashMap<String, DiscoveredPeer>>>,
    receiver_handle: Mutex<Option<JoinHandle<()>>>,
    cleanup_handle: Mutex<Option<JoinHandle<()>>>,
    shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl GlobalDiscoveryReceiver {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<GlobalDiscoveryReceiver> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let std_socket = std::net::UdpSocket::bind(RECEIVER_BIND_ADDR).expect("Failed to bind receiver socket");
            std_socket.set_broadcast(true).expect("Failed set broadcast");

            let socket = Arc::new(UdpSocket::from_std(std_socket).expect("Failed to convert std socket"));

            Self {
                socket,
                peers: Arc::new(Mutex::new(HashMap::new())),
                receiver_handle: Mutex::new(None),
                cleanup_handle: Mutex::new(None),
                shutdown_tx: Mutex::new(None),
            }
        })
    }

    pub fn start(&self, local_name: String, local_ip: String) {
        let mut handle_lock = self.receiver_handle.lock().unwrap();
        if handle_lock.is_some() {
            return;
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        let socket = self.socket.clone();
        let peers_clone = self.peers.clone();

        let receiver_handle = tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            loop {
                tokio::select! {
                    res = socket.recv_from(&mut buf) => {
                        match res {
                            Ok((len, src)) => {
                                if src.ip().to_string() == local_ip {
                                    continue;
                                }

                                if let Ok(heartbeat) = serde_json::from_slice::<HeartbeatPayload>(&buf[..len]) {
                                    if heartbeat.device_name == local_name {
                                        continue;
                                    }

                                    let peer_id = heartbeat.device_id.clone();
                                    let peer_info = NetworkPeerInfo {
                                        id: peer_id.clone(),
                                        name: heartbeat.device_name.clone(),
                                        addr: SocketAddr::new(heartbeat.ip.parse().unwrap(), heartbeat.port),
                                        capabilities: heartbeat.capabilities,
                                        session_id: heartbeat.session_id,
                                        hostname: heartbeat.device_name.clone(),
                                    };

                                    let is_new = {
                                        let mut peers_lock = peers_clone.lock().unwrap();
                                        let is_new = !peers_lock.contains_key(&peer_id);

                                        peers_lock.insert(peer_id.clone(), DiscoveredPeer {
                                            info: peer_info.clone(),
                                            last_seen: Instant::now(),
                                        });

                                        is_new
                                    };

                                    if is_new {
                                        let uniffi_peer = UniffiPeerInfo::from(&peer_info);
                                        info!("Новый пир обнаружен: {:?}", uniffi_peer);

                                        let _ = crate::uniffi_exports::callback_discovery::DISCOVERY_CHANNEL.0.send(
                                            crate::uniffi_exports::callback_discovery::DiscoveryEvent::PeersAdded(vec![uniffi_peer])
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Socket recv error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        let peers = self.peers.clone();
        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
            loop {
                interval.tick().await;
                let now = Instant::now();

                let removed_ids: Vec<String> = {
                    let mut peers_lock = peers.lock().unwrap();
                    let mut removed = Vec::new();

                    peers_lock.retain(|id, peer| {
                        let is_alive = now.duration_since(peer.last_seen) < PEER_TTL;
                        if !is_alive {
                            removed.push(id.clone());
                        }
                        is_alive
                    });

                    removed
                };

                if !removed_ids.is_empty() {
                    info!("пиры на удаление: {:?}", removed_ids);

                    let _ = crate::uniffi_exports::callback_discovery::DISCOVERY_CHANNEL.0.send(
                        crate::uniffi_exports::callback_discovery::DiscoveryEvent::PeersRemoved(removed_ids)
                    );
                }
            }
        });

        *handle_lock = Some(receiver_handle);
        *self.cleanup_handle.lock().unwrap() = Some(cleanup_handle);
    }

    pub fn stop(&self) {
        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.try_send(());
        }
        if let Some(h) = self.receiver_handle.lock().unwrap().take() {
            h.abort();
        }
        if let Some(h) = self.cleanup_handle.lock().unwrap().take() {
            h.abort();
        }
    }

    pub fn get_peers(&self) -> Vec<NetworkPeerInfo> {
        let peers_lock = self.peers.lock().unwrap();
        peers_lock.values().map(|p| p.info.clone()).collect()
    }
}