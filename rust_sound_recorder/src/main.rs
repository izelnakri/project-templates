// TODO:
// Use HW acceleration/openvino for whisper.cpp
// Change /home/izelnakri/Github/whisper.cpp/ to some other location
// Sample wav is not clear sound when run on mpv
use std::{
    fs::create_dir_all,
    process::Stdio,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt}, process::Command, sync::mpsc};

const CHUNK_MS: u64 = 500;

// boot up whisper stream
// listen to whisper stream stdout
// create a stream to write to wav file
// create a stream to push the device sound to whisper stream

#[tokio::main]
async fn main() -> Result<()> {
    let model_path = "/home/izelnakri/Github/whisper.cpp/models/ggml-base.en.bin";
    let wav_path = "samples/izel.wav";
    create_dir_all("samples")?;

    let device = cpal::default_host().default_input_device().expect("No input device");
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    if config.sample_format() != cpal::SampleFormat::F32 {
        panic!("Only f32 format is supported"); // TODO: Check if my format it f32
    }

    println!("Recording from '{}' @ {}Hz {}ch", device.name()?, sample_rate, channels);

    // Start whisper.cpp
    let mut whisper = Command::new("/home/izelnakri/Github/whisper.cpp/build/bin/whisper-stream") 
        .args(["-m", model_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start whisper.cpp");

    let mut whisper_in = whisper.stdin.take().unwrap();
    let mut whisper_out = tokio::io::BufReader::new(whisper.stdout.take().unwrap()).lines();

    // Print transcription
    tokio::spawn(async move {
        while let Ok(Some(line)) = whisper_out.next_line().await {
            println!("📝 {}", line);
        }
    });

    // WAV writer
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let wav_writer = Arc::new(Mutex::new(Some(hound::WavWriter::create(wav_path, spec)?)));

    // Audio chunker
    let (tx, mut rx) = mpsc::channel::<Vec<f32>>(10);

    let tx_stream = tx.clone();
    let writer_clone = wav_writer.clone();
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _| {
            let mut mono = Vec::with_capacity(data.len() / channels as usize);
            for frame in data.chunks(channels as usize) {
                let sum: f32 = frame.iter().copied().sum();
                mono.push(sum / frame.len() as f32);
            }

            if let Ok(mut w) = writer_clone.lock() {
                if let Some(ref mut writer) = *w {
                    for &s in &mono {
                        let _ = writer.write_sample(s);
                    }
                }
            }

            let _ = tx_stream.try_send(mono);
        },
        |e| eprintln!("Stream error: {}", e),
        None,
    )?;
    stream.play()?;

    // Push audio to whisper
    let samples_per_chunk = (sample_rate as f32 * (CHUNK_MS as f32 / 1000.0)) as usize;
    let mut buffer = Vec::with_capacity(samples_per_chunk * 2);
    tokio::spawn(async move {
        while let Some(mono_chunk) = rx.recv().await {
            buffer.extend(mono_chunk);
            while buffer.len() >= samples_per_chunk {
                let chunk: Vec<f32> = buffer.drain(..samples_per_chunk).collect();
                let bytes = bytemuck::cast_slice(&chunk);
                if let Err(e) = whisper_in.write_all(bytes).await {
                    eprintln!("write_all failed: {e}");
                    return;
                }
            }
        }
    });

    println!("🛑 Press Ctrl+C to stop recording");
    tokio::signal::ctrl_c().await?;
    println!("Stopping...");

    stream.pause()?;
    if let Ok(mut writer_guard) = wav_writer.lock() {
        if let Some(writer) = writer_guard.take() {
            writer.finalize()?;
        }
    }

    Ok(())
}
