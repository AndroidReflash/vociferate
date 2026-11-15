use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use crate::database::database::DeviceDatabase;
use crate::uniffi_exports::error::VoiceError;

pub const SETTING_KEY_DEVICE_NAME: &str = "device_name";
pub const INCOMING_CHANNEL_CAPACITY: usize = 16;
const LAN_DETECTION_TARGET: &str = "8.8.8.8:53";

static DEVICE_DB: Mutex<Option<Arc<DeviceDatabase>>> = Mutex::new(None);

pub fn init_device_db(db: DeviceDatabase) {
    let mut guard = DEVICE_DB.lock().unwrap();
    *guard = Some(Arc::new(db));
}

pub fn get_db() -> Result<Arc<DeviceDatabase>, VoiceError> {
    DEVICE_DB
        .lock()
        .unwrap()
        .as_ref()
        .cloned()
        .ok_or_else(|| VoiceError::Other("Database not initialized. Call init_database(path) first.".to_string()))
}

pub fn get_lan_ip() -> Option<IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect(LAN_DETECTION_TARGET).ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}
