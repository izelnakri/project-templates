use std::{
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use v4l::{Format, FourCC, prelude::*, video::Capture};
use v4l::buffer::Type;
use v4l::io::mmap::Stream as MmapStream;
use v4l::io::traits::CaptureStream;  // <-- needed for stream.next()

fn main() -> anyhow::Result<()> {
    let duration = Duration::from_secs(5);
    std::fs::create_dir_all("samples")?;

    let output_path = "samples/final_av.mp4";

    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            // === Input from stdin ===
            "-use_wallclock_as_timestamps", "1", // <--- sync timestamps
            "-f", "rawvideo",
            "-pix_fmt", "yuyv422",    // match webcam pixel format
            "-video_size", "640x480",
            "-framerate", "30",
            "-thread_queue_size", "512",
            "-i", "pipe:0",           // video from stdin, raw frames
            
            // === Audio input ===
            "-thread_queue_size", "512",
            "-f", "alsa",
            "-ac", "2",               // stereo input
            "-ar", "48000",           // 48kHz
            "-i", "default",          // default ALSA input

            // === Hardware encoding with VAAPI ===
            "-vaapi_device", "/dev/dri/renderD128",

            // === Filter graph with hwupload ===
            "-filter_complex", "[0:v]format=nv12,hwupload[vid];[1:a]anull[aud]",

            // === Mapping video/audio from filter graph ===
            "-map", "[vid]",
            "-map", "[aud]",

            "-c:v", "h264_vaapi",
            "-b:v", "5M",               // bit rate (higher = better quality)
            "-maxrate", "5M",
            "-bufsize", "10M",
            "-g", "30",

            // Optional speed-quality balance
            "-qp", "23",
            "-tune", "zerolatency",   // good for streaming + realtime

            "-c:a", "aac",
            "-b:a", "192k",           // higher bitrate for clear speech
            "-shortest",              // stop when shortest stream ends
            "-movflags", "+faststart",// for web streaming
            "-pix_fmt", "yuv420p",    // compatible format

            "-preset", "fast",        // better balance for quality/speed
            "-crf", "23",             // reasonable quality (lower = better)
            output_path,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg");

    let mut stdin = ffmpeg.stdin.take().unwrap();

    let mut dev = Device::new(0).expect("Failed to open video device");
    let format = Format::new(640, 480, FourCC::new(b"YUYV"));
    dev.set_format(&format)?;
    let mut stream = MmapStream::new(&dev, Type::VideoCapture)?;

    let start = Instant::now();
    while Instant::now() - start < duration {
        let (frame, _) = stream.next().unwrap();
        stdin.write_all(frame)?;
    }

    drop(stdin); // signals end of stream to ffmpeg
    let status = ffmpeg.wait()?;
    println!("Saved A/V recording to {}", output_path);
    println!("FFmpeg exited with: {:?}", status);
    Ok(())
}
