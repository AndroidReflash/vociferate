pub mod config;
pub mod encoder;
pub mod decoder;
pub mod error;

pub use config::{CodecConfig, ResampleConfig, FRAME_SIZE_MS, SAMPLE_RATE, CHANNELS, FRAME_SAMPLES};
pub use error::{CodecError, EncoderError, DecoderError, ResampleError};