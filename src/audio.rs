use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioWindow {
    pub rms_dbfs: f32,
}

pub struct AudioCapture {
    _stream: cpal::Stream,
}

impl AudioCapture {
    pub fn new_with_config(sample_rate: u32, window_ms: u64) -> Result<(Self, mpsc::Receiver<AudioWindow>)> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no microphone input device available")?;
        let config = device
            .default_input_config()
            .context("no compatible microphone format available")?;

        let sample_rate = sample_rate.max(8000).min(192000);
        let window_samples = ((sample_rate as u64 * window_ms) / 1000) as usize;
        let (tx, rx) = mpsc::channel();

        let stream_config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let tx_cb = tx.clone();
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            process_audio_frame(data, window_samples, &tx_cb);
                        },
                        move |err| {
                            eprintln!("audio input stream error: {err}");
                        },
                        None,
                    )
                    .context("failed to create microphone input stream")?
            }
            cpal::SampleFormat::I16 => {
                let tx_cb = tx.clone();
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let floats: Vec<f32> = data.iter().map(|sample| *sample as f32 / i16::MAX as f32).collect();
                            process_audio_frame(&floats, window_samples, &tx_cb);
                        },
                        move |err| {
                            eprintln!("audio input stream error: {err}");
                        },
                        None,
                    )
                    .context("failed to create microphone input stream")?
            }
            cpal::SampleFormat::U16 => {
                let tx_cb = tx.clone();
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[u16], _: &cpal::InputCallbackInfo| {
                            let floats: Vec<f32> = data
                                .iter()
                                .map(|sample| (*sample as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0))
                                .collect();
                            process_audio_frame(&floats, window_samples, &tx_cb);
                        },
                        move |err| {
                            eprintln!("audio input stream error: {err}");
                        },
                        None,
                    )
                    .context("failed to create microphone input stream")?
            }
            cpal::SampleFormat::F64 => {
                let tx_cb = tx.clone();
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[f64], _: &cpal::InputCallbackInfo| {
                            let floats: Vec<f32> = data.iter().map(|sample| *sample as f32).collect();
                            process_audio_frame(&floats, window_samples, &tx_cb);
                        },
                        move |err| {
                            eprintln!("audio input stream error: {err}");
                        },
                        None,
                    )
                    .context("failed to create microphone input stream")?
            }
        };

        stream.play().context("failed to start microphone input stream")?;

        Ok((Self { _stream: stream }, rx))
    }
}

fn process_audio_frame(samples: &[f32], window_samples: usize, tx: &std::sync::mpsc::Sender<AudioWindow>) {
    let mut rms_sq_sum = 0f32;
    let mut sample_count = 0usize;

    for sample in samples {
        rms_sq_sum += sample * sample;
        sample_count += 1;

        if sample_count >= window_samples {
            let rms = (rms_sq_sum / sample_count as f32).sqrt();
            let amplitude_db = if rms > 0.0 { 20.0 * rms.log10() } else { -120.0 };
            let rms_dbfs = amplitude_db.clamp(-120.0, 0.0);
            let _ = tx.send(AudioWindow { rms_dbfs });
            sample_count = 0;
            rms_sq_sum = 0.0;
        }
    }
}
