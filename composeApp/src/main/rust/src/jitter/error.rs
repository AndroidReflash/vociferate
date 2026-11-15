use thiserror::Error;

#[derive(Debug, Error)]
pub enum JitterError {
    #[error("Decoder not set")]
    DecoderNotSet,
}