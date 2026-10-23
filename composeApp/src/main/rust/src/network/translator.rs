use bytes::Bytes;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use parking_lot::Mutex as ParkingMutex;
use crate::network::ConnectionManager;
use crate::audio::input::InputShared;
use log::{info, error};
use crate::codec::config::{FRAME_SAMPLES, SAMPLE_RATE, FRAME_SIZE_MS, OPUS_BUF};
use rubato::{FftFixedOut, Resampler};

pub async fn run_audio_broadcaster(
    consumer: Arc<ParkingMutex<Option<rtrb::Consumer<f32>>>>,
    shared: Arc<InputShared>,
    connection_manager: Arc<TokioMutex<ConnectionManager>>,
    sample_rate: u32,
    channels: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let opus_channels = if channels == 1 {
        opus::Channels::Mono
    } else {
        opus::Channels::Stereo
    };

    // Инициализируем Opus с жесткой целевой константой SAMPLE_RATE
    let mut encoder = opus::Encoder::new(
        SAMPLE_RATE,
        opus_channels,
        opus::Application::Voip,
    )?;
    encoder.set_bitrate(opus::Bitrate::Bits(48000))?;

    // Размер выходного фрейма для Opus (под фиксированный SAMPLE_RATE)
    let output_samples_per_channel = (SAMPLE_RATE as usize * FRAME_SIZE_MS as usize) / 1000;
    let total_output_samples = output_samples_per_channel * channels as usize;
    let mut opus_output_buf = vec![0u8; OPUS_BUF.try_into().unwrap()];

    // Создаем ресемплер, если системная частота устройства отличается от целевой SAMPLE_RATE.
    let mut resampler = if sample_rate != SAMPLE_RATE {
        info!("Инициализация ресемплера: {} Hz -> {} Hz (channels: {})", sample_rate, SAMPLE_RATE, channels);

        let temp_resamp = rubato::FftFixedOut::<f32>::new(
            sample_rate as usize,
            SAMPLE_RATE as usize,
            output_samples_per_channel,
            output_samples_per_channel,
            channels as usize,
        )?;
        let input_frames_needed = temp_resamp.input_frames_next();

        Some((temp_resamp, input_frames_needed))
    } else {
        None
    };

    // Строгий интервал отправки фреймов (реальное время аудио)
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(FRAME_SIZE_MS.into()));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    while shared.running.load(Ordering::Relaxed) {
        interval.tick().await;

        // Проверяем, есть ли вообще кому отправлять
        {
            let cm = connection_manager.lock().await;
            if cm.connections.is_empty() {
                continue;
            }
        }

        // Читаем из кольцевого буфера нужное количество семплов
        let pcm_buffer = if let Some((resamp, input_frames_needed)) = &mut resampler {
            // Разыменовываем input_frames_needed (*), так как это &usize
            let total_input_samples = *input_frames_needed * channels as usize;
            let mut input_interleaved = vec![0.0f32; total_input_samples];

            // Читаем из rtrb через pop_chunk или поштучно/итератором, если pop_slice нет
            let read_count = {
                let mut cons_guard = consumer.lock();
                if let Some(cons) = cons_guard.as_mut() {
                    let mut filled = 0;
                    while filled < total_input_samples {
                        if let Ok(sample) = cons.pop() {
                            input_interleaved[filled] = sample;
                            filled += 1;
                        } else {
                            break;
                        }
                    }
                    filled
                } else {
                    0
                }
            };

            // Если буфер не успел налить данные, добиваем тишиной
            if read_count < total_input_samples {
                input_interleaved[read_count..].fill(0.0);
            }

            // Деинтерливим в каналы для rubato
            let mut input_channels = vec![vec![0.0f32; *input_frames_needed]; channels as usize];
            for i in 0..*input_frames_needed {
                for ch in 0..channels as usize {
                    input_channels[ch][i] = input_interleaved[i * channels as usize + ch];
                }
            }

            let mut output_channels = vec![vec![0.0f32; output_samples_per_channel]; channels as usize];
            resamp.process_into_buffer(&input_channels, &mut output_channels, None)?;

            // Интерливим обратно в единый PCM-буфер
            let mut pcm_out = vec![0.0f32; total_output_samples];
            for i in 0..output_samples_per_channel {
                for ch in 0..channels as usize {
                    pcm_out[i * channels as usize + ch] = output_channels[ch][i];
                }
            }
            pcm_out
        } else {
            let mut pcm_out = vec![0.0f32; total_output_samples];

            let read_count = {
                let mut cons_guard = consumer.lock();
                if let Some(cons) = cons_guard.as_mut() {
                    let mut filled = 0;
                    while filled < total_output_samples {
                        if let Ok(sample) = cons.pop() {
                            pcm_out[filled] = sample;
                            filled += 1;
                        } else {
                            break;
                        }
                    }
                    filled
                } else {
                    0
                }
            };

            // Добиваем тишиной при нехватке данных
            if read_count < total_output_samples {
                pcm_out[read_count..].fill(0.0);
            }
            pcm_out
        };

        // Мягкий лимитер (Soft Clipping через tanh вместо жесткого clamp)
        let pcm_normalized: Vec<f32> = pcm_buffer
            .iter()
            .map(|&s| s.tanh())
            .collect();

        // Кодируем в Opus
        let encoded_len = match encoder.encode_float(&pcm_normalized, &mut opus_output_buf) {
            Ok(len) => len,
            Err(e) => {
                error!("Opus encoding error: {}", e);
                continue;
            }
        };

        let opus_bytes = Bytes::copy_from_slice(&opus_output_buf[..encoded_len]);

        // Рассылаем клиентам
        let cm = connection_manager.lock().await;
        for entry in cm.connections.iter() {
            let (peer_id, connection) = entry.pair();
            if let Err(e) = connection.send_audio(opus_bytes.clone()).await {
                error!("Failed to send audio to peer {}: {}", peer_id, e);
            }
        }
    }

    Ok(())
}