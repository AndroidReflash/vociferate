use crate::audio::AudioError;
use crate::discovery::DiscoveryError;
use crate::network::NetworkError;

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
