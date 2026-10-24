use crate::audio::{AudioConfig, AudioInput, AudioOutput};
use crate::codec::config::{CHANNELS, FRAME_SAMPLES, FRAME_SIZE_MS, SAMPLE_RATE, OPUS_BUF};
use crate::codec::decoder::OpusDecoder;
use crate::codec::encoder::OpusEncoder;
use crate::jitter::{JitterBuffer, JitterBufferConfig};
use crate::network::{PeerInfo as NetworkPeerInfo, QuicConnection};
use crate::protocol::Frame;
use crate::uniffi_exports::{PeerInfo, VoiceError};
use std::sync::{Arc, Mutex as StdMutex};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::{mpsc as tokio_mpsc, oneshot, broadcast};
use log::{debug, info};
use tokio::task::JoinHandle;
use dashmap::DashMap;
use crate::protocol::FrameType;
use crate::audio::GlobalAudioOutput;
use crate::audio::input::GlobalAudioInput;
use rubato::{FftFixedIn, Resampler};
use crate::jitter::Packet;
use arc_swap::ArcSwap;
use std::collections::VecDeque;

#[derive(Debug, Clone, uniffi::Record)]
pub struct SessionConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bitrate_bps: i32,
    pub jitter_target_ms: u32,
    pub jitter_max_delay_frames: u32,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            bitrate_bps: 32000,
            jitter_target_ms: 40,
            jitter_max_delay_frames: 8,
            input_device: None,
            output_device: None,
        }
    }
}

struct CaptureTask {
    handle: JoinHandle<()>,
    stop_tx: oneshot::Sender<()>,
}

struct SendTask {
    handle: JoinHandle<()>,
    stop_tx: oneshot::Sender<()>,
}

struct RecvTask {
    handle: JoinHandle<()>,
    stop_tx: oneshot::Sender<()>,
}

struct PlaybackTask {
    handle: JoinHandle<()>,
    stop_tx: oneshot::Sender<()>,
}

pub struct GlobalSession {
    config: SessionConfig,
    encoder: Arc<StdMutex<OpusEncoder>>,
    decoder: Arc<StdMutex<OpusDecoder>>,
    playback_task: StdMutex<Option<PlaybackTask>>,
    jitter_tx: Arc<ArcSwap<crossbeam_channel::Sender<Packet>>>,
    pub connections: DashMap<String, Arc<QuicConnection>>,
    send_tasks: DashMap<String, SendTask>,
    events_tx: tokio_mpsc::UnboundedSender<SessionEvent>,
    ptt_enabled: Arc<StdMutex<bool>>,
    output_device: Arc<StdMutex<Option<String>>>,
    _events_rx: StdMutex<Option<tokio_mpsc::UnboundedReceiver<SessionEvent>>>,
    audio_tx: broadcast::Sender<bytes::Bytes>,
}

impl GlobalSession {
    pub fn global() -> &'static Arc<GlobalSession> {
        static INSTANCE: OnceLock<Arc<GlobalSession>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            log::info!("сессия есть");
            let config = SessionConfig::default();
            let (events_tx, _events_rx) = tokio_mpsc::unbounded_channel();

            let encoder = Arc::new(StdMutex::new(
                OpusEncoder::new(config.bitrate_bps).expect("Global Encoder creation failed")
            ));
            let decoder = Arc::new(StdMutex::new(
                OpusDecoder::new().expect("Global Decoder creation failed")
            ));

            let (jitter_tx, jitter_rx) = crossbeam_channel::unbounded();
            let jitter_tx = Arc::new(ArcSwap::new(Arc::new(jitter_tx)));
            let (audio_tx, _) = broadcast::channel(32);

            let session = Arc::new(Self {
                config,
                encoder,
                decoder,
                playback_task: StdMutex::new(None),
                jitter_tx: jitter_tx,
                connections: DashMap::new(),
                send_tasks: DashMap::new(),
                events_tx,
                ptt_enabled: Arc::new(StdMutex::new(false)),
                output_device: Arc::new(StdMutex::new(None)),
                _events_rx: StdMutex::new(Some(_events_rx)),
                audio_tx,
            });

            session
        })
    }

    pub async fn start(&self, peers: Vec<NetworkPeerInfo>) -> Result<(), String> {
        for peer in peers {
            self.add_peer(peer).await?;
        }
        Ok(())
    }

    pub async fn pub_start_playback(&self) -> Result<(), String> {
        self.stop_playback().await;

        let (jitter_tx, jitter_rx) = crossbeam_channel::unbounded();

        self.jitter_tx.store(Arc::new(jitter_tx));
        self.spawn_playback_task(jitter_rx);

        Ok(())
    }

    pub async fn stop_playback(&self) {
        if let Some(task) = self.playback_task.lock().unwrap().take() {
            let _ = task.stop_tx.send(());
            let _ = task.handle.await;
        }
    }

    pub async fn stop(&self) {
        self.stop_playback().await;
        self.connections.clear();
        self.send_tasks.clear();
    }

    pub async fn add_peer(&self, peer: NetworkPeerInfo) -> Result<(), String> {
        let peer_id = peer.id.clone();

        let connection = self.connections.get(&peer_id)
            .map(|e| e.clone())
            .ok_or_else(|| format!("No connection for peer {peer_id}"))?;

        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        let connection_for_recv = connection.clone();

        let jitter_tx_swap = self.jitter_tx.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    datagram_result = connection_for_recv.connection.read_datagram() => {
                        let data = match datagram_result {
                            Ok(d) => d,
                            Err(_) => break,
                        };

                        match Frame::decode(&data) {
                            Ok(frame) => {
                                if frame.frame_type == FrameType::AudioData {
                                    let tx = jitter_tx_swap.load();
                                    let _ = tx.send(Packet {
                                        seq: frame.sequence,
                                        data: frame.payload.to_vec(),
                                        timestamp_ms: frame.timestamp_ms,
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!("Ошибка декодирования датаграммы: {}", e);
                            }
                        }
                    }
                }
            }
        });

        self.send_tasks.insert(peer_id.clone(), SendTask { handle, stop_tx });
        let _ = self.events_tx.send(SessionEvent::PeerConnected(peer));
        Ok(())
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        if let Some((_, task)) = self.send_tasks.remove(peer_id) {
            let _ = task.stop_tx.send(());
            let _ = task.handle.await;
        }

        self.connections.remove(peer_id);
        let _ = self.events_tx.send(SessionEvent::PeerDisconnected(peer_id.to_string()));
    }

    fn spawn_playback_task(&self, jitter_rx: crossbeam_channel::Receiver<Packet>) {
        let events_tx = self.events_tx.clone();
        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        let jitter_target_ms = self.config.jitter_target_ms;

        let output_device_sample_rate = GlobalAudioOutput::global().current_sample_rate();
        let channels_count = GlobalAudioOutput::global().current_channels() as usize;
        let input_channels_count = 1;
        let jitter_max_ms = 200;

        let handle = tokio::spawn(async move {
            let jitter_config = JitterBufferConfig::new(jitter_max_ms, jitter_target_ms);
            let mut jitter_buffer = JitterBuffer::new(jitter_config);
            let jitter_decoder = OpusDecoder::new().expect("Jitter Decoder creation failed");
            jitter_buffer.set_decoder(jitter_decoder);

            let mut audio_queue = VecDeque::with_capacity(FRAME_SAMPLES * 4);
            let max_allowed_samples = FRAME_SAMPLES * 4; // Лимит очереди, чтобы избежать рассинхрона и гула

            let mut resampler: Option<FftFixedIn<f32>> = None;
            let mut interval = tokio::time::interval(Duration::from_millis(FRAME_SIZE_MS.into()));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                while let Ok(packet) = jitter_rx.try_recv() {
                    let _ = jitter_buffer.push(packet.seq, packet.data, packet.timestamp_ms);
                }

                match jitter_buffer.pop() {
                    Ok(pcm_chunk) => {
                        if !pcm_chunk.is_empty() {
                            audio_queue.extend(pcm_chunk);
                        }
                    }
                    Err(_) => {
                        // Если буфер пуст, ничего страшного, пойдем на тик таймера
                    }
                }

                tokio::select! {
                    _ = &mut stop_rx => {
                        break;
                    },
                    _ = interval.tick() => {
                        let frames = FRAME_SAMPLES;

                        // ЗАЩИТА: Если очередь разрослась из-за clock drift, сбрасываем излишек со старыми данными
                        if audio_queue.len() > max_allowed_samples {
                            let excess = audio_queue.len() - max_allowed_samples;
                            audio_queue.drain(..excess);
                            log::warn!("Audio queue overflow, dropping {} samples to prevent buzzing", excess);
                        }

                        let pcm_filtered = if audio_queue.len() >= frames {
                            let mut chunk: Vec<f32> = audio_queue.drain(..frames).collect();

                            for s in &mut chunk {
                                *s = s.tanh();
                            }
                            chunk
                        } else {
                            // Underrun: данных не хватает, сбрасываем ресемплер во избежание залипания шума
                            resampler = None;
                            vec![0.0f32; frames]
                        };

                        let current_energy: f32 = pcm_filtered.iter().map(|x| x * x).sum::<f32>().sqrt() / (frames as f32).sqrt();
                        let _ = events_tx.send(SessionEvent::LevelOutput(current_energy));

                        let mut input_channels = vec![vec![0.0f32; frames]; input_channels_count];
                        for i in 0..frames {
                            input_channels[0][i] = pcm_filtered[i];
                        }

                        let resampler_to_use = if output_device_sample_rate != SAMPLE_RATE {
                            Some(resampler.get_or_insert_with(|| {
                                rubato::FftFixedIn::<f32>::new(
                                    SAMPLE_RATE as usize,
                                    output_device_sample_rate as usize,
                                    frames,
                                    frames,
                                    input_channels_count,
                                ).unwrap()
                            }))
                        } else {
                            None
                        };

                        let resampled_mono = if let Some(resamp) = resampler_to_use {
                            match resamp.process(&input_channels, None) {
                                Ok(output_channels) => output_channels[0].clone(),
                                Err(e) => {
                                    log::error!("Ошибка ресемплирования вывода: {:?}", e);
                                    resampler = None; // Сбрасываем сломанный ресемплер при ошибке
                                    pcm_filtered.clone()
                                }
                            }
                        } else {
                            pcm_filtered
                        };

                        let out_frames = resampled_mono.len();
                        let mut final_pcm = vec![0.0f32; out_frames * channels_count];

                        for i in 0..out_frames {
                            let sample = resampled_mono[i].clamp(-0.99, 0.99);
                            for ch in 0..channels_count {
                                final_pcm[i * channels_count + ch] = sample;
                            }
                        }

                        GlobalAudioOutput::global().write(&final_pcm);
                    }
                }
            }
        });

        *self.playback_task.lock().unwrap() = Some(PlaybackTask { handle, stop_tx });
    }

    pub fn set_ptt(&self, enabled: bool) {
        *self.ptt_enabled.lock().unwrap() = enabled;
    }

    pub fn set_output_device(&self, device: Option<String>) {
        *self.output_device.lock().unwrap() = device;
    }

    pub fn set_connection(&self, peer_id: String, connection: Arc<QuicConnection>) {
        self.connections.insert(peer_id, connection);
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SessionStats {
    pub jitter_ms: f32,
    pub underruns: u64,
    pub overruns: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct SessionHandle {
    pub inner: u64,
}

#[derive(Clone, Debug)]
pub enum SessionEvent {
    PeerConnected(NetworkPeerInfo),
    PeerDisconnected(String),
    LevelInput(f32),
    LevelOutput(f32),
    Stats(SessionStats),
    Error(String),
}


