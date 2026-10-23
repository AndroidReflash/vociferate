use cpal::{SampleRate, StreamConfig};
use uniffi;

#[derive(Debug, Clone, uniffi::Record)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 1,
            buffer_size: 960,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub max_input_channels: u16,
    pub max_output_channels: u16,
    pub default_sample_rate: u32,
    pub default_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
#[uniffi(with_try_read)]
pub enum AudioError {
    NoDevice { reason: String },
    DeviceUnavailable { reason: String },
    StreamError { reason: String },
    FormatNotSupported { reason: String },
    Cpal { reason: String },
    DeviceDisconnected { device_id: String },
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::NoDevice { reason } => write!(f, "No audio device: {reason}"),
            AudioError::DeviceUnavailable { reason } => write!(f, "Device unavailable: {reason}"),
            AudioError::StreamError { reason } => write!(f, "Stream error: {reason}"),
            AudioError::FormatNotSupported { reason } => write!(f, "Format not supported: {reason}"),
            AudioError::Cpal { reason } => write!(f, "CPAL error: {reason}"),
            AudioError::DeviceDisconnected { device_id } => write!(f, "Device disconnected: {device_id}"),
        }
    }
}
impl std::error::Error for AudioError {}
