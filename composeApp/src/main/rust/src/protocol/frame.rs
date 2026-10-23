use crate::protocol::handshake::HandshakePayload;
use crate::protocol::error::{ProtocolError, Result};
use bytes::{BytesMut};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const PROTOCOL_VERSION: u8 = 1;
pub const MAGIC_BYTES: [u8; 4] = *b"VOIC";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FrameType {
    Handshake = 0x01,
    HandshakeAck = 0x02,
    AudioData = 0x10,
    AudioAck = 0x11,
    Ping = 0x20,
    Pong = 0x21,
    Goodbye = 0xFF,
}

impl TryFrom<u8> for FrameType {
    type Error = ProtocolError;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(FrameType::Handshake),
            0x02 => Ok(FrameType::HandshakeAck),
            0x10 => Ok(FrameType::AudioData),
            0x11 => Ok(FrameType::AudioAck),
            0x20 => Ok(FrameType::Ping),
            0x21 => Ok(FrameType::Pong),
            0xFF => Ok(FrameType::Goodbye),
            _ => Err(ProtocolError::UnknownFrameType(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub frame_type: FrameType,
    pub sequence: u64,
    pub timestamp_ms: u64,
    pub payload: bytes::Bytes,
}

impl Frame {
    pub fn new(frame_type: FrameType, sequence: u64, payload: bytes::Bytes) -> Self {
        let timestamp_ms = current_time_ms();
        Self {
            frame_type,
            sequence,
            timestamp_ms,
            payload,
        }
    }

    pub fn handshake(seq: u64, payload: crate::protocol::handshake::HandshakePayload) -> Result<Self> {
        let bytes = bincode::serialize(&payload)
            .map_err(|e| ProtocolError::Serialize(format!("Handshake serialize: {e}")))?;
        Ok(Self::new(FrameType::Handshake, seq, bytes::Bytes::from(bytes)))
    }

    pub fn audio_data(seq: u64, opus_data: bytes::Bytes) -> Self {
        Self::new(FrameType::AudioData, seq, opus_data)
    }

    pub fn ping(seq: u64) -> Self {
        Self::new(FrameType::Ping, seq, bytes::Bytes::new())
    }

    pub fn pong(seq: u64) -> Self {
        Self::new(FrameType::Pong, seq, bytes::Bytes::new())
    }

    pub fn goodbye(seq: u64) -> Self {
        Self::new(FrameType::Goodbye, seq, bytes::Bytes::new())
    }

    pub fn encode(&self) -> Result<bytes::Bytes> {
        let mut buf = BytesMut::with_capacity(1 + 8 + 8 + self.payload.len() + 4);
        buf.extend_from_slice(&MAGIC_BYTES);
        buf.extend_from_slice(&[PROTOCOL_VERSION]);
        buf.extend_from_slice(&[self.frame_type as u8]);
        buf.extend_from_slice(&self.sequence.to_be_bytes());
        buf.extend_from_slice(&self.timestamp_ms.to_be_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.payload);
        Ok(buf.freeze())
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 4 + 1 + 1 + 8 + 8 + 4 {
            return Err(ProtocolError::FrameTooShort);
        }

        if &data[0..4] != MAGIC_BYTES {
            return Err(ProtocolError::InvalidMagicBytes);
        }

        let version = data[4];
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::VersionMismatch { expected: PROTOCOL_VERSION, got: version });
        }

        let frame_type = FrameType::try_from(data[5])?;
        let sequence = u64::from_be_bytes(data[6..14].try_into().unwrap());
        let timestamp_ms = u64::from_be_bytes(data[14..22].try_into().unwrap());
        let payload_len = u32::from_be_bytes(data[22..26].try_into().unwrap()) as usize;

        if data.len() < 26 + payload_len {
            return Err(ProtocolError::PayloadTruncated);
        }

        let payload = bytes::Bytes::copy_from_slice(&data[26..26 + payload_len]);

        Ok(Self {
            frame_type,
            sequence,
            timestamp_ms,
            payload,
        })
    }

    pub fn decode_handshake(&self) -> Result<crate::protocol::handshake::HandshakePayload> {
        if self.frame_type != FrameType::Handshake {
            return Err(ProtocolError::NotHandshakeFrame);
        }
        bincode::deserialize(&self.payload)
            .map_err(|e| ProtocolError::Deserialize(format!("Handshake deserialize: {e}")))
    }

    pub fn decode_handshake_ack(&self) -> Result<crate::protocol::handshake::HandshakePayload> {
        if self.frame_type != FrameType::HandshakeAck {
            return Err(ProtocolError::NotHandshakeAckFrame);
        }
        bincode::deserialize(&self.payload)
            .map_err(|e| ProtocolError::Deserialize(format!("HandshakeAck deserialize: {e}")))
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frame(type={:?}, seq={}, ts={}ms, payload={}B)",
            self.frame_type, self.sequence, self.timestamp_ms, self.payload.len()
        )
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}