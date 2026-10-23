use super::config::types::{AudioConfig, AudioError};
use super::config::stream_config::{build_stream_config, find_compatible_config};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream};
use parking_lot::Mutex as ParkingMutex;
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use uniffi;
use crate::network::ConnectionManager;
use crate::uniffi_exports::{SHARED_CONNECTIONS};
use crate::uniffi_exports::VoiceError;
use crate::network::translator::run_audio_broadcaster;
use log::{info, error};
use std::sync::{OnceLock, Mutex};

const RING_CAPACITY: usize = 8192;

pub struct InputShared {
    level: AtomicU32,
    pub running: AtomicBool,
    device_name: String,
}

#[derive(uniffi::Object, Clone)]
pub struct InputControl {
    shared: Arc<InputShared>,
    consumer: Arc<ParkingMutex<Option<Consumer<f32>>>>,
}

#[uniffi::export]
impl InputControl {
    pub fn current_level(&self) -> f32 {
        f32::from_bits(self.shared.level.load(Ordering::Relaxed))
    }

    pub fn is_running(&self) -> bool {
        self.shared.running.load(Ordering::Relaxed)
    }

    pub fn device_name(&self) -> String {
        self.shared.device_name.clone()
    }

    pub fn available_samples(&self) -> u32 {
        RING_CAPACITY as u32
    }

    pub fn read(&self, count: u32) -> Vec<f32> {
        let consumer = {
            let mut guard = self.consumer.lock();
            guard.take()
        };

        let mut result = Vec::with_capacity(count as usize);
        if let Some(mut consumer) = consumer {
            for _ in 0..count {
                match consumer.pop() {
                    Ok(sample) => result.push(sample),
                    Err(_) => break,
                }
            }
            *self.consumer.lock() = Some(consumer);
        }
        result
    }
}

struct AudioInputInner {
    stream: Option<SendableStream>,
    control: InputControl,
}

#[derive(uniffi::Object)]
pub struct AudioInput {
    inner: Arc<ParkingMutex<AudioInputInner>>,
}

struct SendableStream(Stream);

unsafe impl Send for SendableStream {}
unsafe impl Sync for SendableStream {}

#[uniffi::export]
impl AudioInput {
    #[uniffi::constructor]
    pub fn new(device_id: Option<String>, config: AudioConfig) -> Result<Self, AudioError> {
        let host = cpal::default_host();

        let device = if let Some(ref id) = device_id {
            host.input_devices()
                .map_err(|e| AudioError::Cpal { reason: e.to_string() })?
                .find(|d| d.name().unwrap_or_default() == *id)
                .ok_or_else(|| {
                    log::error!("Device with ID '{}' not found among available inputs", id);
                    AudioError::DeviceUnavailable { reason: id.clone() }
                })?
        } else {
            host.default_input_device()
                .ok_or_else(|| {
                    log::error!("No default input device found by host");
                    AudioError::NoDevice { reason: "No default input device".into() }
                })?
        };

        let device_name = device.name().unwrap_or_else(|_| "unknown".into());

        let supported_config = find_compatible_config(&device, &config, "input")?;
        let stream_config = build_stream_config(&config, &supported_config);

        let (producer, consumer) = RingBuffer::new(RING_CAPACITY);
        let producer = Arc::new(ParkingMutex::new(Some(producer)));
        let consumer = Arc::new(ParkingMutex::new(Some(consumer)));

        let shared = Arc::new(InputShared {
            level: AtomicU32::new(0),
            running: AtomicBool::new(false),
            device_name: device_name.clone(),
        });
        let control = InputControl {
            shared: shared.clone(),
            consumer: consumer.clone(),
        };
        let shared_for_err = shared.clone();
        let shared_for_stream = shared.clone();
        let producer_for_stream = producer.clone();
        let value = device_name.clone();
        let err_fn = move |err| {
            log::error!("Audio input stream error on device '{}': {}", value, err);
            shared_for_err.running.store(false, Ordering::Relaxed);
        };

        // цикл, что плюётся звуком
        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !shared_for_stream.running.load(Ordering::Relaxed) {
                    return;
                }

                let mut prod_guard = producer_for_stream.lock();
                if let Some(prod) = prod_guard.as_mut() {
                    let mut peak: f32 = 0.0;
                    for &sample in data {
                        peak = peak.max(sample.abs());
                        let _ = prod.push(sample);
                    }
                    shared_for_stream.level.store(peak.to_bits(), Ordering::Relaxed);
                }
            },
            err_fn,
                None,
            ).map_err(|e| {
                log::error!("Failed to build input stream: {}", e);
                AudioError::StreamError { reason: e.to_string() }
        })?;

        let consumer_clone = control.consumer.clone();
        let shared_clone = Arc::clone(&shared);

        let connection_manager_arc = {
            let shared_conn = SHARED_CONNECTIONS.lock().unwrap();
            match shared_conn.as_ref() {
                Some(cm) => cm.clone(), // Это Arc<ConnectionManager> (или Arc<Mutex<ConnectionManager>>)
                None => {
                    return Err(AudioError::DeviceUnavailable {
                        reason: "Discovery not started. Call start_discovery() first.".to_string()
                    });
                }
            }
        };
        let connection_manager_clone = Arc::clone(&connection_manager_arc);

        let sample_rate = stream_config.sample_rate.0;
        let channels = stream_config.channels;

        shared.running.store(true, Ordering::Relaxed);

        tokio::spawn(async move {
            if let Err(e) = run_audio_broadcaster(
                consumer_clone,
                shared_clone,
                connection_manager_arc,
                sample_rate,
                channels,
            ).await {
                log::error!("Audio broadcaster loop crashed: {}", e);
            }
        });

        let inner = AudioInputInner {
            stream: Some(SendableStream(stream)),
            control,
        };
        Ok(Self {
            inner: Arc::new(ParkingMutex::new(inner)),
        })
    }

    pub fn start(&self) -> Result<(), AudioError> {
        let inner = self.inner.lock();
        inner.control.shared.running.store(true, Ordering::Relaxed);
        if let Some(stream) = &inner.stream {
            stream.0.play().map_err(|e| AudioError::StreamError { reason: e.to_string() })?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<(), AudioError> {
        let inner = self.inner.lock();
        inner.control.shared.running.store(false, Ordering::Relaxed);
        if let Some(stream) = &inner.stream {
            stream.0.pause().map_err(|e| AudioError::StreamError { reason: e.to_string() })?;
        }
        Ok(())
    }

    pub fn control(&self) -> InputControl {
        self.inner.lock().control.clone()
    }
}

pub struct GlobalAudioInput {
    pub current_control: Mutex<Option<InputControl>>,
    current_config: Mutex<Option<AudioConfig>>,
    active_input: Mutex<Option<AudioInput>>,
}

impl GlobalAudioInput {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<GlobalAudioInput> = OnceLock::new();
        INSTANCE.get_or_init(|| Self {
            current_control: Mutex::new(None),
            current_config: Mutex::new(None),
            active_input: Mutex::new(None),
        })
    }

    pub fn set_device(&self, device_id: Option<String>, config: AudioConfig) -> Result<(), String> {
        let input = AudioInput::new(device_id, config.clone())
            .map_err(|e| format!("Failed to create AudioInput: {e:?}"))?;

        input.start().map_err(|e| format!("Failed to start AudioInput: {e:?}"))?;
        let control = input.control();

        let mut ctrl_lock = self.current_control.lock().unwrap();
        *ctrl_lock = Some(control);

        let mut in_lock = self.active_input.lock().unwrap();
        *in_lock = Some(input);

        let mut cfg_lock = self.current_config.lock().unwrap();
        *cfg_lock = Some(config);

        Ok(())
    }

    pub fn start(&self) -> Result<(), String> {
        let ctrl_lock = self.current_control.lock().unwrap();
        if let Some(control) = ctrl_lock.as_ref() {
            control.shared.running.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let ctrl_lock = self.current_control.lock().unwrap();
        if let Some(control) = ctrl_lock.as_ref() {
            control.shared.running.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }
}