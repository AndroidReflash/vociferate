use crate::codec::decoder::OpusDecoder;
use crate::jitter::error::JitterError;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Packet {
    pub seq: u64,
    pub data: Vec<u8>,
    pub timestamp_ms: u64,
}

pub struct JitterBuffer {
    decoder: Option<OpusDecoder>,
    queue: VecDeque<Packet>,
    max_capacity: usize,
}

impl JitterBuffer {
    pub fn new(_config: crate::jitter::config::JitterBufferConfig) -> Self {
        let max_capacity = 5;
        Self {
            decoder: None,
            queue: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn set_decoder(&mut self, decoder: OpusDecoder) {
        self.decoder = Some(decoder);
    }

    pub fn push(&mut self, seq: u64, data: Vec<u8>, timestamp_ms: u64) -> Result<(), JitterError> {
        if self.queue.len() >= self.max_capacity {
            self.queue.pop_front();
        }
        self.queue.push_back(Packet { seq, data, timestamp_ms });
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Vec<f32>, JitterError> {
        let decoder = self.decoder.as_mut().ok_or(JitterError::DecoderNotSet)?;

        let pcm = if let Some(packet) = self.queue.pop_front() {
            match decoder.decode(&packet.data) {
                Ok(decoded_vec) => decoded_vec,
                Err(e) => {
                    log::error!("ОШИБКА ДЕКОДИРОВАНИЯ OPUS: {:?}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        Ok(pcm)
    }
}