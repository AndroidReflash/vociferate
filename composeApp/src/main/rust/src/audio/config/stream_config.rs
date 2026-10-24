use cpal::traits::DeviceTrait;
use cpal::{Device, SampleRate, StreamConfig, SupportedStreamConfig};
use super::types::{AudioConfig, AudioError};

pub fn build_stream_config(_config: &AudioConfig, device_config: &SupportedStreamConfig) -> StreamConfig {
    let mut stream_config = device_config.config();
    stream_config.buffer_size = cpal::BufferSize::Default;
    stream_config
}

pub fn find_compatible_config(device: &Device, config: &AudioConfig, kind: &str) -> Result<SupportedStreamConfig, AudioError> {
    if kind == "input" {
        let default_config = device.default_input_config()
            .map_err(|e| AudioError::Cpal { reason: e.to_string() })?;

        log::info!("Используем чистый дефолт input: каналы={}, частота={}, формат={:?}",
            default_config.channels(),
            default_config.sample_rate().0,
            default_config.sample_format()
        );

        return Ok(default_config)
    } else {
        let default_config = device.default_output_config()
            .map_err(|e| AudioError::Cpal { reason: e.to_string() })?;

        log::info!("Используем чистый дефолт1 output: каналы={}, частота={}, формат={:?}",
            default_config.channels(),
            default_config.sample_rate().0,
            default_config.sample_format()
        );

        return Ok(default_config);
    }

    Err(AudioError::FormatNotSupported {
        reason: format!("No compatible config for {}ch @ {}Hz", config.channels, config.sample_rate),
    })
}