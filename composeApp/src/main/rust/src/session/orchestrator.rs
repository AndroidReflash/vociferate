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
use std::collections::{HashMap, VecDeque};

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

struct SendTask {
    handle: JoinHandle<()>,
    stop_tx: oneshot::Sender<()>,
}

struct PlaybackTask {
    handle: JoinHandle<()>,
    stop_tx: oneshot::Sender<()>,
}

struct PeerStream {
    packet_tx: crossbeam_channel::Sender<Packet>,
    packet_rx: StdMutex<Option<crossbeam_channel::Receiver<Packet>>>,
}

pub struct GlobalSession {
    config: SessionConfig,
    encoder: Arc<StdMutex<OpusEncoder>>,
    decoder: Arc<StdMutex<OpusDecoder>>,
    playback_task: StdMutex<Option<PlaybackTask>>,
    peer_streams: Arc<DashMap<String, PeerStream>>,
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

            let peer_streams = Arc::new(DashMap::new());
            let (audio_tx, _) = broadcast::channel(32);

            let session = Arc::new(Self {
                config,
                encoder,
                decoder,
                playback_task: StdMutex::new(None),
                peer_streams,
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
        self.spawn_playback_task();
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
        self.peer_streams.clear();
    }

    pub async fn add_peer(&self, peer: NetworkPeerInfo) -> Result<(), String> {
        let peer_id = peer.id.clone();

        let connection = self.connections.get(&peer_id)
            .map(|e| e.clone())
            .ok_or_else(|| format!("No connection for peer {peer_id}"))?;

        let (packet_tx, packet_rx) = crossbeam_channel::unbounded::<Packet>();
        self.peer_streams.insert(peer_id.clone(), PeerStream {
            packet_tx,
            packet_rx: StdMutex::new(Some(packet_rx))
        });

        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        let connection_for_recv = connection.clone();
        let peer_id_for_task = peer_id.clone();
        let peer_streams = self.peer_streams.clone();

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
                                    // Шлем в канал конкретного пира
                                    if let Some(stream) = peer_streams.get(&peer_id_for_task) {
                                        let _ = stream.packet_tx.send(Packet {
                                            seq: frame.sequence,
                                            data: frame.payload.to_vec(),
                                            timestamp_ms: frame.timestamp_ms,
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Ошибка декодирования датаграммы от {}: {}", peer_id_for_task, e);
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
        self.peer_streams.remove(peer_id);
        self.connections.remove(peer_id);
        let _ = self.events_tx.send(SessionEvent::PeerDisconnected(peer_id.to_string()));
    }

    fn spawn_playback_task(&self) {
        let events_tx = self.events_tx.clone();
        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        let jitter_target_ms = self.config.jitter_target_ms;
        let peer_streams = self.peer_streams.clone();

        let output_device_sample_rate = GlobalAudioOutput::global().current_sample_rate();
        let channels_count = GlobalAudioOutput::global().current_channels() as usize;
        let input_channels_count = 1;
        let jitter_max_ms = 200;

        let handle = tokio::spawn(async move {
            let mut peer_buffers: HashMap<String, (JitterBuffer, crossbeam_channel::Receiver<Packet>)> = HashMap::new();
            let mut resampler: Option<FftFixedIn<f32>> = None;

            let mut interval = tokio::time::interval(Duration::from_millis(FRAME_SIZE_MS.into()));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = &mut stop_rx => {
                        break;
                    },
                    _ = interval.tick() => {
                        let frames = FRAME_SAMPLES;

                        peer_buffers.retain(|id, _| peer_streams.contains_key(id));

                        for entry in peer_streams.iter() {
                            let peer_id = entry.key().clone();
                            if !peer_buffers.contains_key(&peer_id) {
                                if let Some(mut rx_lock) = entry.value().packet_rx.lock().unwrap().take() {
                                    let jitter_config = JitterBufferConfig::new(jitter_max_ms, jitter_target_ms);
                                    let mut jb = JitterBuffer::new(jitter_config);
                                    jb.set_decoder(OpusDecoder::new().expect("Peer Decoder creation failed"));

                                    peer_buffers.insert(peer_id, (jb, rx_lock));
                                }
                            }
                        }

                        for (_, (buffer, rx)) in peer_buffers.iter_mut() {
                            while let Ok(packet) = rx.try_recv() {
                                let _ = buffer.push(packet.seq, packet.data, packet.timestamp_ms);
                            }
                        }

                        let mut mixed_pcm = vec![0.0f32; frames];
                        let mut active_speakers = 0;

                        for (_, (buffer, _)) in peer_buffers.iter_mut() {
                            if let Ok(pcm_chunk) = buffer.pop() {
                                if !pcm_chunk.is_empty() && pcm_chunk.len() == frames {
                                    active_speakers += 1;
                                    for i in 0..frames {
                                        mixed_pcm[i] += pcm_chunk[i];
                                    }
                                }
                            }
                        }

                        if active_speakers > 0 {
                            let norm_factor = 1.0 / (active_speakers as f32).sqrt();
                            for s in &mut mixed_pcm {
                                *s = (*s * norm_factor).clamp(-0.99, 0.99);
                            }
                        }

                        let current_energy: f32 = mixed_pcm.iter().map(|x| x * x).sum::<f32>().sqrt() / (frames as f32).sqrt();
                        let _ = events_tx.send(SessionEvent::LevelOutput(current_energy));

                        let mut input_channels = vec![vec![0.005f32; frames]; input_channels_count];
                        for i in 0..frames {
                            input_channels[0][i] = mixed_pcm[i];
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
                                    log::error!("Ошибка ресемплирования вывода микшера: {:?}", e);
                                    resampler = None;
                                    mixed_pcm.clone()
                                }
                            }
                        } else {
                            mixed_pcm
                        };

                        let out_frames = resampled_mono.len();
                        let mut final_pcm = vec![0.0f32; out_frames * channels_count];

                        let mut has_audio = false;
                        for i in 0..out_frames {
                            let sample = resampled_mono[i].clamp(-0.99, 0.99);
                            if sample != 0.0 {
                                has_audio = true;
                            }
                            for ch in 0..channels_count {
                                final_pcm[i * channels_count + ch] = sample;
                            }
                        }

                        if has_audio {
                            GlobalAudioOutput::global().write(&final_pcm);
                        }
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