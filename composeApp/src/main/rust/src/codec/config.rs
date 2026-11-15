pub const FRAME_SIZE_MS: u32 = 40;
pub const SAMPLE_RATE: u32 = 48000;
pub const CHANNELS: u16 = 1;
pub const FRAME_SAMPLES: usize = (SAMPLE_RATE as usize * FRAME_SIZE_MS as usize) / 1000;
pub const OPUS_BUF: i32 = 4000;
