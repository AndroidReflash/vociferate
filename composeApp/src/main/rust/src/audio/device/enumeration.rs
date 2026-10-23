use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;
use crate::audio::config::types::{AudioDevice, AudioError};
use log::{info};
use std::collections::HashSet;

// getMinBufferSize(): Invalid channel configuration
fn device_to_audio_device(device: &Device, is_default: bool, kind: &str) -> Result<AudioDevice, AudioError> {
    let name = device.name().unwrap_or_else(|_| "unknown".into());
    let config_result = if kind == "input" {
        device.default_input_config()
    } else {
        device.default_output_config()
    };

    let config = config_result.ok();
    let (max_in, max_out) = if kind == "input" {
        (config.as_ref().map(|c| c.channels()).unwrap_or(0), 0)
    } else {
        (0, config.as_ref().map(|c| c.channels()).unwrap_or(0))
    };
    let default_sr = config.as_ref().map(|c| c.sample_rate().0).unwrap_or(48000);
    let default_fmt = config.as_ref().map(|c| format!("{:?}", c.sample_format())).unwrap_or_else(|| "unknown".into());

    Ok(AudioDevice {
        id: name.clone(),
        name,
        is_default,
        max_input_channels: max_in,
        max_output_channels: max_out,
        default_sample_rate: default_sr,
        default_format: default_fmt,
    })
}

pub fn list_input_devices() -> Result<Vec<AudioDevice>, AudioError> {
    let host = cpal::default_host();
    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());
    let mut devices = Vec::new();
    let mut seen_names = HashSet::new();

    for device in host.input_devices().map_err(|e| AudioError::Cpal { reason: e.to_string() })? {
        let name = device.name().unwrap_or_default();
        if !seen_names.insert(name.clone()) {
            continue;
        }
        let is_default = default_name.as_ref().map(|n| n == &name).unwrap_or(false);
        devices.push(device_to_audio_device(&device, is_default, "input")?);
    }
    Ok(devices)
}

pub fn list_output_devices() -> Result<Vec<AudioDevice>, AudioError> {
    let host = cpal::default_host();
    let default_device = host.default_output_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());

    let mut devices = Vec::new();
    let mut seen_names = HashSet::new(); // Сет для дедупликации

    for device in host.output_devices().map_err(|e| AudioError::Cpal { reason: e.to_string() })? {
        let name = device.name().unwrap_or_default();

        if !seen_names.insert(name.clone()) {
            continue;
        }

        let is_default = default_name.as_ref().map(|n| n == &name).unwrap_or(false);
        devices.push(device_to_audio_device(&device, is_default, "output")?);
    }

    Ok(devices)
}

pub fn default_input_device() -> Result<AudioDevice, AudioError> {
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or_else(|| AudioError::NoDevice { reason: "No default input device".into() })?;
    device_to_audio_device(&device, true, "input")
}

pub fn default_output_device() -> Result<AudioDevice, AudioError> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or_else(|| AudioError::NoDevice { reason: "No default output device".into() })?;
    device_to_audio_device(&device, true, "output")
}