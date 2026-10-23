//! Audio UniFFI exports

use crate::audio::{list_input_devices, list_output_devices, AudioConfig, AudioInput, AudioError, InputControl, AudioDevice};
use crate::uniffi_exports::VoiceError;
use std::sync::{Arc, LazyLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use log::{info, error};
use crate::audio::GlobalAudioOutput;
use crate::audio::input::GlobalAudioInput;
use crate::session::GlobalSession;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

struct ListeningTask {
    control: InputControl,
}

static LISTENING_TASKS: LazyLock<Mutex<HashMap<u64, ListeningTask>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static NEXT_LISTENING_HANDLE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, uniffi::Record)]
pub struct ListeningHandle {
    pub inner: u64,
}

#[uniffi::export]
pub fn listening_handle_control(handle: ListeningHandle) -> InputControl {
    let guard = LISTENING_TASKS.lock().unwrap();
    guard.get(&handle.inner).map(|t| t.control.clone())
        .expect("ListeningHandle not found")
}

#[uniffi::export]
pub fn listening_handle_is_running(handle: ListeningHandle) -> bool {
    let guard = LISTENING_TASKS.lock().unwrap();
    guard.get(&handle.inner).map(|t| t.control.is_running()).unwrap_or(false)
}

#[uniffi::export]
pub fn listening_handle_current_level(handle: ListeningHandle) -> f32 {
    let guard = LISTENING_TASKS.lock().unwrap();
    guard.get(&handle.inner).map(|t| t.control.current_level()).unwrap_or(0.0)
}

#[uniffi::export]
pub fn list_audio_input_devices() -> Result<Vec<AudioDevice>, VoiceError> {
    Ok(list_input_devices()?)
}

#[uniffi::export]
pub fn list_audio_output_devices() -> Result<Vec<AudioDevice>, VoiceError> {
    Ok(list_output_devices()?)
}

// включить микрофон - эндпоинт для котлин
#[uniffi::export]
pub fn start_listening_micro(
    device_id: Option<String>,
    config: AudioConfig,
) -> Result<(), VoiceError> {
    RUNTIME.block_on(async move {
        let _ = GlobalAudioInput::global().stop();
        let _ = GlobalAudioInput::global()
            .set_device(device_id, config)
            .map_err(|e| VoiceError::Audio(e))?;
        let _ = GlobalAudioInput::global().start();
        Ok(())
    })
}

#[uniffi::export]
pub fn stop_listening_micro() -> Result<(), VoiceError> {
    RUNTIME.block_on(async move {
        let _ = GlobalAudioInput::global().stop();
        Ok(())
    })
}

// включение динамиков
#[uniffi::export]
pub fn start_speakers(
    device_id: Option<String>, // речь про айди источника звука
    config: AudioConfig,
) -> Result<ListeningHandle, VoiceError> {
    RUNTIME.block_on(async move {
          let _ = GlobalAudioOutput::global().stop();

          let _ = GlobalAudioOutput::global()
              .set_device(device_id, config)
              .map_err(|e| VoiceError::Audio(e))?;

          let _ = GlobalAudioOutput::global().start();
          let _ = GlobalSession::global().pub_start_playback().await;

          let inner = NEXT_LISTENING_HANDLE.fetch_add(1, Ordering::Relaxed);
          Ok(ListeningHandle { inner })
    })
}

// отключение динамиков
#[uniffi::export]
pub fn stop_speakers() -> Result<(), VoiceError>{
    RUNTIME.block_on(async move {
        let _ = GlobalAudioOutput::global().stop();
        let _ = GlobalSession::global().stop_playback().await;
        Ok(())
    })
}