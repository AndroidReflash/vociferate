use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Opus decoder creation failed: {0}")]
    Creation(String),

    #[error("Decoding failed: {0}")]
    Decode(String),
}
