use crate::protocol::capabilities::Capabilities;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    pub protocol_version: u8,
    pub device_name: String,
    pub capabilities: Capabilities,
    pub session_id: u64,
    pub device_id: String,
}