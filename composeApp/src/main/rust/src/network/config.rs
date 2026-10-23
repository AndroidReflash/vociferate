use uniffi;
use std::sync::OnceLock;
use uuid::Uuid;

static DEVICE_ID: OnceLock<String> = OnceLock::new();

#[derive(Debug, Clone, uniffi::Record)]
pub struct NetworkConfig {
    pub alpn: String,
    pub idle_timeout_ms: u64,
    pub keep_alive_interval_ms: u64,
    pub max_concurrent_streams: u32,
    pub initial_rtt_ms: u64,
    pub max_datagram_size: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            alpn: "vociferate/1".into(),
            idle_timeout_ms: 15000,
            keep_alive_interval_ms: 1000,
            max_concurrent_streams: 10,
            initial_rtt_ms: 100,
            max_datagram_size: 1200,
        }
    }
}

impl NetworkConfig {
    pub fn new(alpn: String) -> Self {
        Self {
            alpn,
            ..Default::default()
        }
    }

    pub fn with_idle_timeout(mut self, ms: u64) -> Self {
        self.idle_timeout_ms = ms;
        self
    }

    pub fn with_keep_alive(mut self, ms: u64) -> Self {
        self.keep_alive_interval_ms = ms;
        self
    }
}

pub fn get_cached_device_id() -> &'static str {
    DEVICE_ID.get_or_init(|| Uuid::new_v4().to_string())
}

pub fn init_device_id(stored_id: Option<String>) {
    let id = stored_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let _ = DEVICE_ID.set(id);
}