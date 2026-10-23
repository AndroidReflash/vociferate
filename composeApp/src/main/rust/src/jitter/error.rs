use thiserror::Error;

#[derive(Debug, Error)]
pub enum JitterError {
    #[error("Buffer full: capacity {capacity} exceeded")]
    BufferFull { capacity: usize },

    #[error("Invalid sequence number: expected >= {expected}, got {actual}")]
    InvalidSequence { expected: u64, actual: u64 },

    #[error("Packet too old: seq {seq}, current base {base}")]
    PacketTooOld { seq: u64, base: u64 },

    #[error("Packet too new: seq {seq}, max ahead {max_ahead}")]
    PacketTooNew { seq: u64, max_ahead: u64 },

    #[error("Buffer empty")]
    Empty,

    #[error("Decoder not set")]
    DecoderNotSet,

    #[error("Decoder error: {0}")]
    Decoder(String),

    #[error("Internal error: {0}")]
    Internal(String),
}