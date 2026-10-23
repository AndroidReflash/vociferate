use crate::codec::config::{FRAME_SAMPLES, SAMPLE_RATE};
use crate::codec::error::EncoderError;
use opus::Encoder as OpusEncoderInner;
use opus::{Application, Channels};
use log::{info};

pub struct OpusEncoder {
    encoder: OpusEncoderInner,
    bitrate_bps: i32,
}

impl OpusEncoder {
    pub fn new(bitrate_bps: i32) -> Result<Self, EncoderError> {
        let mut encoder = OpusEncoderInner::new(SAMPLE_RATE, Channels::Mono, Application::Voip)
            .map_err(|e| EncoderError::Creation(e.to_string()))?;
        encoder.set_bitrate(opus::Bitrate::Bits(bitrate_bps))
            .map_err(|e| EncoderError::Creation(e.to_string()))?;
        encoder.set_inband_fec(true)
            .map_err(|e| EncoderError::Creation(e.to_string()))?;
        encoder.set_packet_loss_perc(20)
            .map_err(|e| EncoderError::Creation(e.to_string()))?;
        encoder.set_dtx(true)
            .map_err(|e| EncoderError::Creation(e.to_string()))?;
        Ok(Self { encoder, bitrate_bps })
    }
}