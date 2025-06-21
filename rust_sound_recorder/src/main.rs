use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::Duration,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let duration = Duration::from_secs(5);

    std::fs::create_dir_all("samples")?;
    let path = "samples/izel.wav";

    // CPAL Setup
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = device.default_input_config()?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    println!(
        "Recording: {:?} - {}Hz, {}ch, {:?} for {} seconds",
        device.name()?,
        sample_rate,
        channels,
        config.sample_format(),
        duration.as_secs()
    );

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let writer = hound::WavWriter::create(path, spec)?;
    let writer = Arc::new(std::sync::Mutex::new(Some(writer)));
    let stop_flag = Arc::new(AtomicBool::new(false));

    let writer_clone = writer.clone();
    let stop_flag_clone = stop_flag.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                if stop_flag_clone.load(Ordering::Relaxed) {
                    return;
                }
                let mut writer_guard = writer_clone.lock().unwrap();
                if let Some(writer) = writer_guard.as_mut() {
                    for &sample in data {
                        if let Err(e) = writer.write_sample(sample) {
                            eprintln!("Write error: {}", e);
                        }
                    }
                }
            },
            err_fn,
            None,
        )?,
        _ => panic!("Unsupported sample format (must be f32)"),
    };

    stream.play()?;
    sleep(duration).await;
    stop_flag.store(true, Ordering::Relaxed);
    stream.pause()?;

    let mut writer_guard = writer.lock().unwrap();
    if let Some(writer) = writer_guard.take() {
        writer.finalize()?;
    }

    println!("Saved high-res recording to {}", path);
    Ok(())
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Stream error: {}", err);
}
