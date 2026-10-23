use uniffi;

pub const FRAME_SIZE_MS: u32 = 40;
pub const SAMPLE_RATE: u32 = 48000;
pub const CHANNELS: u16 = 1;
pub const FRAME_SAMPLES: usize = (SAMPLE_RATE as usize * FRAME_SIZE_MS as usize) / 1000;
pub const OPUS_BUF: i32 = 4000;

#[derive(Debug, Clone, uniffi::Record)]
pub struct CodecConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub frame_size_ms: u32,
    pub bitrate_bps: i32,
    pub enable_fec: bool,
    pub enable_dtx: bool,
    pub packet_loss_perc: u8,
}

impl Default for CodecConfig {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            frame_size_ms: FRAME_SIZE_MS,
            bitrate_bps: 32000,
            enable_fec: true,
            enable_dtx: true,
            packet_loss_perc: 10,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ResampleConfig {
    pub in_rate: u32,
    pub out_rate: u32,
    pub channels: u16,
    pub frames_per_chunk: u32,
}

impl ResampleConfig {
    pub fn new(in_rate: u32, out_rate: u32, channels: u16, frames_per_chunk: u32) -> Self {
        Self {
            in_rate,
            out_rate,
            channels,
            frames_per_chunk,
        }
    }
}