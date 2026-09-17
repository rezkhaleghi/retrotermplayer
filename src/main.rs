use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let url = std::env::args()
        .nth(1)
        .expect("Usage: cargo run -- <youtube-url>");

    println!("Getting YouTube stream...");

    let output = Command::new("yt-dlp")
        .args([
            "-f",
            "worstvideo",
            "-g",
            &url,
        ])
        .output()
        .expect("Failed to run yt-dlp");

    if !output.status.success() {
        eprintln!(
            "yt-dlp failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stream_url = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    println!("Starting terminal video...");
    thread::sleep(Duration::from_secs(1));

    let width = 80;
    let height = 30;

    let frame_size = width * height;

    let charset: &[u8] = b"@%#*+=-:. ";

    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-loglevel",
            "quiet",

            "-i",
            &stream_url,

            // Resize before sending frames to Rust.
            "-vf",
            "scale=80:30,fps=10,format=gray",

            "-f",
            "rawvideo",
            "-pix_fmt",
            "gray",

            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg");

    let mut stdout = ffmpeg
        .stdout
        .take()
        .expect("Failed to access ffmpeg stdout");

    // Clear screen.
    print!("\x1b[2J");

    // Hide cursor.
    print!("\x1b[?25l");

    // Make sure cursor comes back even when the program exits.
    let result = render_video(
        &mut stdout,
        width,
        height,
        frame_size,
        charset,
    );

    // Show cursor.
    print!("\x1b[?25h");

    // Move cursor below the video.
    print!("\x1b[{};1H", height + 2);

    io::stdout().flush().unwrap();

    if let Err(error) = result {
        eprintln!("Playback error: {error}");
    }

    let _ = ffmpeg.wait();
}

fn render_video(
    stdout: &mut impl Read,
    width: usize,
    height: usize,
    frame_size: usize,
    charset: &[u8],
) -> io::Result<()> {
    let mut frame = vec![0u8; frame_size];

    let frame_duration = Duration::from_millis(100);

    loop {
        let frame_start = Instant::now();

        let mut read = 0;

        while read < frame_size {
            let n = stdout.read(&mut frame[read..])?;

            if n == 0 {
                return Ok(());
            }

            read += n;
        }

        // Move cursor to the top-left.
        print!("\x1b[H");

        let mut output = String::with_capacity(
            width * height + height,
        );

        for y in 0..height {
            for x in 0..width {
                let pixel = frame[y * width + x];

                let index =
                    pixel as usize * (charset.len() - 1) / 255;

                output.push(charset[index] as char);
            }

            output.push('\n');
        }

        print!("{output}");

        io::stdout().flush()?;

        // Don't render faster than 10 FPS.
        let elapsed = frame_start.elapsed();

        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }
}