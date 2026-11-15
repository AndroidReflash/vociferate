use std::net::{IpAddr, SocketAddr};

use crate::network::PeerInfo as NetworkPeerInfo;
use crate::uniffi_exports::error::VoiceError;

#[derive(Debug, Clone, uniffi::Record)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub hostname: String,
    pub is_active_micro: bool,
}

impl PeerInfo {
    pub fn socket_addr(&self) -> Result<SocketAddr, VoiceError> {
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
            is_active_micro: peer.is_active_micro,
        }
    }
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct SessionHandle {
    pub inner: u64,
}
