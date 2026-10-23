#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Frame too short")]
    FrameTooShort,
    #[error("Invalid magic bytes")]
    InvalidMagicBytes,
    #[error("Protocol version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u8, got: u8 },
    #[error("Unknown frame type: {0}")]
    UnknownFrameType(u8),
    #[error("Frame payload truncated")]
    PayloadTruncated,
    #[error("Not a handshake frame")]
    NotHandshakeFrame,
    #[error("Not a handshake ack frame")]
    NotHandshakeAckFrame,
    #[error("Serialization failed: {0}")]
    Serialize(String),
    #[error("Deserialization failed: {0}")]
    Deserialize(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;