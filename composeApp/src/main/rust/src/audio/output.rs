use super::config::types::{AudioConfig, AudioError};
use super::config::stream_config::{build_stream_config, find_compatible_config};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream};
use parking_lot::Mutex as ParkingMutex;
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use uniffi;
use log::info;

const RING_CAPACITY: usize = 8192;

struct OutputShared {
    level: AtomicU32,
    running: AtomicBool,
    underruns: AtomicU64,
    device_name: String,
}

#[derive(uniffi::Object, Clone)]
pub struct OutputControl {
    shared: Arc<OutputShared>,
    producer: Arc<ParkingMutex<Option<Producer<f32>>>>,
}

#[uniffi::export]
impl OutputControl {
    pub fn current_level(&self) -> f32 {
        f32::from_bits(self.shared.level.load(Ordering::Relaxed))
    }

    pub fn underruns(&self) -> u64 {
        self.shared.underruns.load(Ordering::Relaxed)
    }

    pub fn is_running(&self) -> bool {
        self.shared.running.load(Ordering::Relaxed)
    }

    pub fn device_name(&self) -> String {
        self.shared.device_name.clone()
    }

    pub fn free_capacity(&self) -> u32 {
        self.producer.lock().as_ref().map(|p| p.slots() as u32).unwrap_or(0)
    }

    pub fn write(&self, buffer: &[f32]) -> u32 {
        if let Some(producer) = self.producer.lock().as_mut() {
            let (pushed, _) = producer.push_partial_slice(buffer);
            pushed.len() as u32
        } else {
            0
        }
    }
}

struct AudioOutputInner {
    stream: Option<SendableStream>,
    control: OutputControl,
}

#[derive(uniffi::Object)]
pub struct AudioOutput {
    inner: Arc<ParkingMutex<AudioOutputInner>>,
}

struct SendableStream(Stream);

unsafe impl Send for SendableStream {}
unsafe impl Sync for SendableStream {}

#[uniffi::export]
impl AudioOutput {
    #[uniffi::constructor]
    pub fn new(device_id: Option<String>, config: AudioConfig) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = if let Some(id) = device_id {
            host.output_devices().map_err(|e| AudioError::Cpal { reason: e.to_string() })?
                .find(|d| d.name().unwrap_or_default() == id)
                .ok_or_else(|| AudioError::DeviceUnavailable { reason: id })?
        } else {
            host.default_output_device()
                .ok_or_else(|| AudioError::NoDevice { reason: "No default output device".into() })?
        };
        log::info!("Выводим на {:?}", device.name());
        let device_name = device.name().unwrap_or_else(|_| "unknown".into());
        let supported_config = find_compatible_config(&device, &config, "output")?;
        let stream_config = build_stream_config(&config, &supported_config);

        log::info!("Конфигурация вывода: каналы={}, частота={}, формат={:?}, буфер={:?}",
            stream_config.channels,
            stream_config.sample_rate.0,
            supported_config.sample_format(),
            stream_config.buffer_size
        );
        log::error!("сам синглтон");
        GlobalAudioOutput::global().update_stream_info(stream_config.sample_rate.0, stream_config.channels);

        let (producer, consumer) = RingBuffer::new(RING_CAPACITY);
        let producer = Arc::new(ParkingMutex::new(Some(producer)));
        let consumer = Arc::new(ParkingMutex::new(Some(consumer)));
        let shared = Arc::new(OutputShared {
            level: AtomicU32::new(0),
            running: AtomicBool::new(false),
            underruns: AtomicU64::new(0),
            device_name: device_name.clone(),
        });
        let control = OutputControl {
            shared: shared.clone(),
            producer: producer.clone(),
        };

        let shared_for_err = shared.clone();
        let shared_for_stream = shared.clone();
        let consumer_for_stream = consumer.clone();
        let err_fn = move |err| {
            log::error!("Audio output stream error: {}", err);
            shared_for_err.running.store(false, Ordering::Relaxed);
        };
        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if !shared_for_stream.running.load(Ordering::Relaxed) {
                    data.fill(0.0);
                    return;
                }
                // достаём звук для аудиокарты пачкой вместо забора по одному
                // нет, я не ебу схуяли так было
                if let Some(cons) = consumer_for_stream.lock().as_mut() {
                    let (filled, _unfilled) = cons.pop_partial_slice(data);
                    let filled_len = filled.len();
                    if filled_len > 0 {
                        if filled_len < data.len() {
                            data[filled_len..].fill(0.0);
                        }
                        let peak = data[..filled_len].iter().fold(0.0f32, |max, &s| max.max(s.abs()));
                        shared_for_stream.level.store(peak.to_bits(), Ordering::Relaxed);
                    } else {
                        data.fill(0.0);
                        shared_for_stream.underruns.fetch_add(1, Ordering::Relaxed);
                        shared_for_stream.level.store(0.0f32.to_bits(), Ordering::Relaxed);
                    }
                } else {
                    data.fill(0.0);
                }
            },
            err_fn,
            None,
        ).map_err(|e| AudioError::StreamError { reason: e.to_string() })?;
        let inner = AudioOutputInner {
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

    pub fn control(&self) -> OutputControl {
        self.inner.lock().control.clone()
    }
}

// глобальный синглтон для вывода звука
pub struct GlobalAudioOutput {
    current_control: Mutex<Option<OutputControl>>,
    current_config: Mutex<Option<AudioConfig>>,
    active_output: Mutex<Option<AudioOutput>>,
    current_sample_rate: Mutex<u32>,
    current_channels: Mutex<u16>,
}

impl GlobalAudioOutput {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<GlobalAudioOutput> = OnceLock::new();
        INSTANCE.get_or_init(|| Self {
            current_control: Mutex::new(None),
            current_config: Mutex::new(None),
            active_output: Mutex::new(None),
            current_sample_rate: Mutex::new(48000), // дефолт
            current_channels: Mutex::new(2),        // дефолт
        })
    }

    pub fn update_stream_info(&self, sample_rate: u32, channels: u16) {
        *self.current_sample_rate.lock().unwrap() = sample_rate;
        *self.current_channels.lock().unwrap() = channels;
    }

    // Смена устройства или конфигурации на лету (синглтон сам пересоздаст поток под капотом)
    pub fn set_device(&self, device_id: Option<String>, config: AudioConfig) -> Result<(), String> {
        let output = AudioOutput::new(device_id, config.clone())
            .map_err(|e| format!("Failed to create AudioOutput: {e:?}"))?;

        output.start().map_err(|e| format!("Failed to start AudioOutput: {e:?}"))?;
        let control = output.control();
        log::error!("ставим устройство вывода");
        let mut ctrl_lock = self.current_control.lock().unwrap();
        *ctrl_lock = Some(control);

        let mut out_lock = self.active_output.lock().unwrap();
        *out_lock = Some(output);

        let mut cfg_lock = self.current_config.lock().unwrap();
        *cfg_lock = Some(config);

        Ok(())
    }

    pub fn current_sample_rate(&self) -> u32 {
        *self.current_sample_rate.lock().unwrap()
    }

    pub fn current_channels(&self) -> u16 {
        *self.current_channels.lock().unwrap()
    }

    pub fn write(&self, pcm: &[f32]) {
        let mut ctrl_lock = self.current_control.lock().unwrap();
        if let Some(control) = ctrl_lock.as_mut() {
            control.write(pcm);
        }
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
