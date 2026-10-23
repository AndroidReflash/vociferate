use crate::codec::config::{FRAME_SAMPLES, SAMPLE_RATE};
use crate::codec::error::DecoderError;
use opus::Decoder as OpusDecoderInner;
use opus::Channels;

pub struct OpusDecoder {
    decoder: OpusDecoderInner,
}

impl OpusDecoder {
    pub fn new() -> Result<Self, DecoderError> {
        let decoder = OpusDecoderInner::new(SAMPLE_RATE, Channels::Mono)
            .map_err(|e| DecoderError::Creation(e.to_string()))?;
        Ok(Self { decoder })
    }

    pub fn decode(&mut self, opus_data: &[u8]) -> Result<Vec<f32>, DecoderError> {
        let mut pcm_out = vec![0.0f32; 5760];

        let samples_decoded = self.decoder.decode_float(opus_data, &mut pcm_out, false)
            .map_err(|e| DecoderError::Decode(e.to_string()))?;

        pcm_out.truncate(samples_decoded);

        Ok(pcm_out)
    }

    pub fn decode_plc(&mut self) -> Result<Vec<f32>, DecoderError> {
        let mut pcm_out = vec![0.0f32; FRAME_SAMPLES]; // стандартный размер для PLC
        let samples_decoded = self.decoder.decode_float(&[], &mut pcm_out, false)
            .map_err(|e| DecoderError::Decode(e.to_string()))?;

        pcm_out.truncate(samples_decoded);
        Ok(pcm_out)
    }

    pub fn frame_samples(&self) -> usize {
        FRAME_SAMPLES
    }

    pub fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    pub fn channels(&self) -> u16 {
        1
    }

    pub fn reset_state(&mut self) {
        self.decoder.reset_state().ok();
    }
}

impl Default for OpusDecoder {
    fn default() -> Self {
        Self::new().expect("OpusDecoder creation failed")
    }
}