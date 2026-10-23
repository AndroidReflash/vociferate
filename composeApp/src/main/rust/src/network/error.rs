use crate::protocol::ProtocolError;
use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("QUIC error: {0}")]
    Quic(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("Timeout")]
    Timeout,

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Peer rejected: {0}")]
    PeerRejected(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ProtocolError> for NetworkError {
    fn from(err: ProtocolError) -> Self {
        NetworkError::Protocol(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, NetworkError>;