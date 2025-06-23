// NOTE: This is VAAPI(Amd+Intel) based, not Intel Quick Sync
use std::{
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use v4l::{Format, FourCC, prelude::*, video::Capture};
use v4l::buffer::Type;
use v4l::io::mmap::Stream as MmapStream;
use v4l::io::traits::CaptureStream;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all("samples")?;

    let duration = Duration::from_secs(5);
    let output_path = "samples/final_av.mp4";

    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output files without asking.

            // === Video input from stdin ===
            "-use_wallclock_as_timestamps", "1", // <--- sync timestamps
            "-f", "rawvideo",
            "-pixel_format", "yuyv422",          // <--- crucial: match webcam pixel format
            "-video_size", "640x480",
            "-framerate", "30",                  // Solid framerate for smoothness, 60+ fps is for sports, gaming
            "-thread_queue_size", "512",         // This was done for video/audio sync for hw acceleration
            "-i", "pipe:0",                       // video from stdin, raw frames

            // === Audio input ===
            "-thread_queue_size", "512",         // This prevents frame drops by a pre-process buffer
            "-f", "alsa",
            "-ac", "2",                           // stereo input
            "-ar", "48000",                       // 48kHz
            "-i", "default",                      // default ALSA input

            // === Hardware encoding with VAAPI ===
            "-vaapi_device", "/dev/dri/renderD128",

            // === Filter graph (video hwupload, passthrough audio) ===
            "-filter_complex", "[0:v]format=nv12,hwupload[vid];[1:a]anull[aud]", // sync audio

            // === Mapping filtered streams ===
            "-map", "[vid]",
            "-map", "[aud]",

            // === Video codec & rate control ===
            "-c:v", "h264_vaapi",                 // encoding
            "-b:v", "5M",                         // bit rate (higher = better quality)
            "-maxrate", "5M",
            "-bufsize", "10M",                    // This controls variation, 2x more is good practice
            "-g", "30",                           // 1 keyframe every 30 frames
            "-compression_level", "4",            // good for streaming + realtime for VAAPI

            // === Audio codec & bitrate ===
            "-c:a", "aac",                        // audio codec, aac great, opus an option
            "-b:a", "256k",                       // audio compression, higher bitrate for clear speech(64-256k)

            // === Output format and compatibility ===
            "-shortest",                          // stop when shortest stream ends
            "-movflags", "+faststart",            // for web streaming

            // === Performance tuning ===
            "-pix_fmt", "vaapi",                  // ensure compatibility with VAAPI encoder

            // === Output path ===
            output_path,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start ffmpeg")?;

    let mut stdin = ffmpeg.stdin.take().context("Failed to open ffmpeg stdin")?;

    let dev = Device::new(0).context("Failed to open video device")?;
    let format = Format::new(640, 480, FourCC::new(b"YUYV"));
    dev.set_format(&format).context("Failed to set webcam format")?;
    let mut stream = MmapStream::new(&dev, Type::VideoCapture)
        .context("Failed to initialize mmap stream")?;

    let start = Instant::now();
    while Instant::now() - start < duration {
        let (frame, _) = stream.next().context("Failed to read next frame")?;
        stdin.write_all(frame)?;
    }

    drop(stdin); // signals end of stream to ffmpeg
    let status = ffmpeg.wait()?;
    println!("Saved A/V recording to {}", output_path);
    println!("FFmpeg exited with: {:?}", status);
    Ok(())
}
