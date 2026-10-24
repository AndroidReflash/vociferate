use crate::codec::decoder::OpusDecoder;
use crate::codec::config::{FRAME_SAMPLES, SAMPLE_RATE, FRAME_SIZE_MS, OPUS_BUF};
use crate::jitter::config::{JitterBufferConfig};
use crate::jitter::error::JitterError;
use std::collections::VecDeque;
use crate::audio::GlobalAudioOutput;

#[derive(Clone)]
pub struct Packet {
    pub seq: u64,
    pub data: Vec<u8>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct JitterStats {
    pub jitter_ms: f32,
    pub underruns: u64,
    pub overruns: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub packets_discarded: u64,
    pub current_delay_frames: u32,
    pub target_delay_frames: u32,
}

pub struct JitterBuffer {
    config: JitterBufferConfig,
    decoder: Option<OpusDecoder>,
    pub stats: JitterStats,
    queue: VecDeque<Packet>,
    max_capacity: usize,
}

impl JitterBuffer {
    pub fn new(config: JitterBufferConfig) -> Self {
        // Допустим, FRAME_SIZE_MS = 20мс. Для 100мс задержки нужно 5 кадров.
        let max_capacity = 5;
        Self {
            config,
            decoder: None,
            stats: JitterStats::default(),
            queue: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn with_decoder(config: JitterBufferConfig, decoder: OpusDecoder) -> Self {
        let mut s = Self::new(config);
        s.decoder = Some(decoder);
        s
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

    pub fn buffered_frames(&self) -> usize {
        self.queue.len()
    }

    pub fn stats(&self) -> JitterStats {
        self.stats.clone()
    }

    pub fn reset(&mut self) {
        self.queue.clear();
        self.stats = JitterStats::default();
    }

    pub fn clear(&mut self) {
        self.reset();
    }

    pub fn config(&self) -> &JitterBufferConfig {
        &self.config
    }
}