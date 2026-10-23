pub mod audio;
pub mod database;
pub mod discovery;
pub mod network;
pub mod callback;
pub mod callback_discovery;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use crate::audio::{AudioDevice, AudioError};
use crate::discovery::{DiscoveryError, MdnsService};
use crate::network::{
    ConnectionManager, NetworkConfig, NetworkError, PeerInfo as NetworkPeerInfo,
    QuicConnection,
};
use crate::protocol::Capabilities;
use crate::session::{SessionConfig, SessionEvent, SessionHandle, SessionStats};
use crate::database::database::{DeviceDatabase, DbError};

use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

// Database constants
const SETTING_KEY_DEVICE_NAME: &str = "device_name";
const LAN_DETECTION_TARGET: &str = "8.8.8.8:53";
const INCOMING_CHANNEL_CAPACITY: usize = 16;

fn get_lan_ip() -> Option<IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect(LAN_DETECTION_TARGET).ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum VoiceError {
    #[error("Audio: {0}")]
    Audio(String),

    #[error("Network: {0}")]
    Network(String),

    #[error("Discovery: {0}")]
    Discovery(String),

    #[error("Session: {0}")]
    Session(String),

    #[error("Unknown session handle: {0}")]
    UnknownSession(u64),

    #[error("Not a valid ip address: {0}")]
    InvalidAddress(String),

    #[error("Other: {0}")]
    Other(String),
}

impl From<AudioError> for VoiceError {
    fn from(err: AudioError) -> Self {
        VoiceError::Audio(err.to_string())
    }
}

impl From<NetworkError> for VoiceError {
    fn from(err: NetworkError) -> Self {
        VoiceError::Network(err.to_string())
    }
}

impl From<DiscoveryError> for VoiceError {
    fn from(err: DiscoveryError) -> Self {
        VoiceError::Discovery(err.to_string())
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub hostname: String,
}

impl PeerInfo {
    fn socket_addr(&self) -> Result<SocketAddr, VoiceError> {
        let ip: IpAddr = self
            .ip
            .parse()
            .map_err(|_| VoiceError::InvalidAddress(self.ip.clone()))?;
        Ok(SocketAddr::new(ip, self.port))
    }
}

impl From<&NetworkPeerInfo> for PeerInfo {
    fn from(peer: &NetworkPeerInfo) -> Self {
        Self {
            id: peer.id.clone(),
            name: peer.name.clone(),
            ip: peer.addr.ip().to_string(),
            port: peer.addr.port(),
            hostname: peer.hostname.clone(),
        }
    }
}

pub static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for VoiceTransfer core")
});

pub static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);




pub static SHARED_CONNECTIONS: LazyLock<Mutex<Option<Arc<AsyncMutex<ConnectionManager>>>>> =
    LazyLock::new(|| Mutex::new(None));


static DEVICE_DB: Mutex<Option<std::sync::Arc<DeviceDatabase>>> = Mutex::new(None);



fn get_db() -> Result<std::sync::Arc<DeviceDatabase>, VoiceError> {
    DEVICE_DB.lock().unwrap().as_ref().cloned()
        .ok_or(VoiceError::Other("Database not initialized. Call init_database(path) first.".to_string()))
}