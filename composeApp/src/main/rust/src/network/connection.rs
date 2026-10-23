use crate::network::config::NetworkConfig;
use crate::network::error::{NetworkError, Result};
use crate::protocol::{Frame, FrameType, HandshakePayload};
use bytes::Bytes;
use quinn::{Connection, RecvStream, SendStream};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;
use log::{error, debug, info};

pub struct QuicConnection {
    pub connection: Connection,
    recv_stream: tokio::sync::Mutex<Option<quinn::RecvStream>>,
    send_stream: tokio::sync::Mutex<Option<quinn::SendStream>>,
    config: NetworkConfig,
    sequence: std::sync::atomic::AtomicU32,
    handshake_done: bool,
    peer_info: Option<HandshakePayload>,
}

impl QuicConnection {
    pub fn new(connection: Connection, config: NetworkConfig) -> Self {
        Self {
            connection,
            send_stream: tokio::sync::Mutex::new(None),
            recv_stream: tokio::sync::Mutex::new(None),
            config,
            sequence: std::sync::atomic::AtomicU32::new(0),
            handshake_done: false,
            peer_info: None,
        }
    }

    pub async fn accept_bi_stream(&mut self) -> Result<(SendStream, RecvStream)> {
        let (send, recv) = self
            .connection
            .accept_bi()
            .await
            .map_err(|e| NetworkError::Stream(format!("Accept bi-stream failed: {e}")))?;
        log::info!("SUCCESS: Incoming bi-stream accepted from peer!");
        Ok((send, recv))
    }

    pub async fn open_bi_stream(&mut self) -> Result<(SendStream, RecvStream)> {
        let (send, recv) = self
            .connection
            .open_bi()
            .await
            .map_err(|e| NetworkError::Stream(format!("Open bi-stream failed: {e}")))?;
        Ok((send, recv))
    }

    pub async fn perform_handshake(
        &mut self,
        local_payload: HandshakePayload,
    ) -> Result<HandshakePayload> {
        let (mut send, mut recv) = self.open_bi_stream().await?;
        let handshake = Frame::handshake(self.next_seq(), local_payload)?;
        let encoded = handshake.encode()?;
        send.write_all(&encoded)
            .await
            .map_err(|e| NetworkError::Stream(format!("Handshake send failed: {e}")))?;
        send.finish()
            .map_err(|e| NetworkError::Stream(format!("Handshake finish failed: {e}")))?;
        let response_bytes = recv
            .read_to_end(usize::MAX)
            .await
            .map_err(|e| NetworkError::Stream(format!("Handshake recv failed: {e}")))?;
        let response = Frame::decode(&response_bytes)?;
        if response.frame_type != FrameType::HandshakeAck {
            return Err(NetworkError::Protocol(
                "Expected HandshakeAck".into(),
            ));
        }

        let ack_payload = response.decode_handshake_ack()?;
        self.handshake_done = true;
        self.peer_info = Some(ack_payload.clone());
        Ok(ack_payload)
    }

    pub async fn perform_handshake_server(
        &mut self,
        local_payload: HandshakePayload,
    ) -> Result<HandshakePayload> {
        log::info!("хэндшейк случился");
        let (mut send, mut recv) = self.accept_bi_stream().await?;
        let received = recv
            .read_to_end(usize::MAX)
            .await
            .map_err(|e| NetworkError::Stream(format!("Handshake recv failed: {e}")))?;
        let request = Frame::decode(&received)?;
        if request.frame_type != FrameType::Handshake {
            return Err(NetworkError::Protocol(
                "Expected Handshake".into(),
            ));
        }
        let peer_payload = request.decode_handshake()?;
        let ack_frame = Frame {
            frame_type: FrameType::HandshakeAck,
            sequence: self.next_seq(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            payload: {
                let bytes = bincode::serialize(&local_payload)
                    .map_err(|e| NetworkError::Protocol(format!("HandshakeAck serialize: {e}")))?;
                Bytes::from(bytes)
            },
        };
        let encoded = ack_frame.encode()?;
        send.write_all(&encoded)
            .await
            .map_err(|e| NetworkError::Stream(format!("HandshakeAck send failed: {e}")))?;
        send.finish()
            .map_err(|e| NetworkError::Stream(format!("HandshakeAck finish failed: {e}")))?;
        self.handshake_done = true;
        self.peer_info = Some(peer_payload.clone());
        Ok(peer_payload)
    }
    // отправление - нужен переводчик (translator.rs)
    pub async fn send_audio(&self, opus_data: Bytes) -> Result<()> {
       if !self.handshake_done {
           return Err(NetworkError::InvalidState(
               "Handshake not completed".into(),
           ));
       }

       let frame = Frame::audio_data(self.next_seq(), opus_data);
       let encoded = frame.encode()?;

       // Убираем .freeze(), шлем сам encoded (если это Bytes или преобразуется в Bytes/Vec<u8> по типу quinn)
       self.connection.send_datagram(encoded)
           .map_err(|e| NetworkError::Stream(format!("Audio datagram send failed: {e}")))?;

       Ok(())
    }

    fn next_seq(&self) -> u64 {
        self.sequence.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u64
    }
}
