use crate::audio::config::types::AudioError;

pub fn error_from_build_stream(err: String) -> AudioError {
    AudioError::StreamError { reason: err }
}

pub fn error_from_play_stream(err: String) -> AudioError {
    AudioError::StreamError { reason: err }
}

pub fn error_from_pause_stream(err: String) -> AudioError {
    AudioError::StreamError { reason: err }
}

pub fn error_from_devices(err: String) -> AudioError {
    AudioError::Cpal { reason: err }
}