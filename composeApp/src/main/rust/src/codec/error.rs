use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Encoder error: {0}")]
    Encoder(#[from] EncoderError),

    #[error("Decoder error: {0}")]
    Decoder(#[from] DecoderError),

    #[error("Resample error: {0}")]
    Resample(#[from] ResampleError),
}

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Opus encoder creation failed: {0}")]
    Creation(String),

    #[error("Encoding failed: {0}")]
    Encode(String),

    #[error("Invalid input size: expected {expected}, got {actual}")]
    InvalidInputSize { expected: usize, actual: usize },
}

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Opus decoder creation failed: {0}")]
    Creation(String),

    #[error("Decoding failed: {0}")]
    Decode(String),

    #[error("Invalid output buffer size: expected {expected}, got {actual}")]
    InvalidOutputSize { expected: usize, actual: usize },
}

#[derive(Debug, Error)]
pub enum ResampleError {
    #[error("Resampler creation failed: {0}")]
    Creation(String),

    #[error("Resampling failed: {0}")]
    Process(String),

    #[error("Invalid input size: expected {expected}, got {actual}")]
    InvalidInputSize { expected: usize, actual: usize },

    #[error("Channel count mismatch: resampler has {expected}, input has {actual}")]
    ChannelMismatch { expected: usize, actual: usize },
}