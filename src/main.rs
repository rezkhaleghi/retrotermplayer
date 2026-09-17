use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const WIDTH: usize = 100;
const HEIGHT: usize = 60;
const FPS: u64 = 15;

fn main() {
    let youtube_url = std::env::args()
        .nth(1)
        .expect("Usage: retrotermplayer <youtube-url>");

    println!("Getting YouTube stream...");

    let stream_url = Command::new("yt-dlp")
        .args(["-f", "worstvideo", "-g", &youtube_url])
        .output()
        .expect("Failed to run yt-dlp");

    if !stream_url.status.success() {
        eprintln!("yt-dlp failed");
        return;
    }

    let stream_url = String::from_utf8_lossy(&stream_url.stdout)
        .trim()
        .to_string();

    if stream_url.is_empty() {
        eprintln!("Could not get stream URL");
        return;
    }

    println!("Starting retro terminal video...");

    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-loglevel",
            "quiet",
            "-i",
            &stream_url,
            "-vf",
            &format!(
                "scale={}:{}:force_original_aspect_ratio=decrease,\
pad={}:{}:(ow-iw)/2:(oh-ih)/2,\
fps={},format=gray",
                WIDTH,
                HEIGHT,
                WIDTH,
                HEIGHT,
                FPS
            ),
            "-f",
            "rawvideo",
            "-pix_fmt",
            "gray",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start ffmpeg");

    let mut stdout = ffmpeg
        .stdout
        .take()
        .expect("Failed to capture ffmpeg output");

    print!("\x1b[2J\x1b[H\x1b[?25l");
    io::stdout().flush().unwrap();

    let result = render_video(&mut stdout);

    print!("\x1b[?25h\x1b[0m\n");
    io::stdout().flush().unwrap();

    if let Err(error) = result {
        eprintln!("Rendering error: {error}");
    }

    let _ = ffmpeg.wait();
}

fn render_video<R: Read>(reader: &mut R) -> io::Result<()> {
    let frame_size = WIDTH * HEIGHT;
    let mut frame = vec![0u8; frame_size];

    let frame_duration = Duration::from_millis(1000 / FPS);

    loop {
        let frame_start = Instant::now();

        if !read_exact_frame(reader, &mut frame)? {
            break;
        }

        render_frame(&frame);

        io::stdout().flush()?;

        let elapsed = frame_start.elapsed();

        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    Ok(())
}

fn read_exact_frame<R: Read>(
    reader: &mut R,
    buffer: &mut [u8],
) -> io::Result<bool> {
    let mut offset = 0;

    while offset < buffer.len() {
        match reader.read(&mut buffer[offset..])? {
            0 => {
                return Ok(false);
            }
            bytes_read => {
                offset += bytes_read;
            }
        }
    }

    Ok(true)
}

fn render_frame(frame: &[u8]) {
    let mut output = String::new();

    // Move cursor to top-left without clearing the terminal.
    output.push_str("\x1b[H");

    // Each terminal character represents TWO vertical pixels.
    for y in (0..HEIGHT).step_by(2) {
        for x in 0..WIDTH {
            let top = frame[y * WIDTH + x];

            let bottom = if y + 1 < HEIGHT {
                frame[(y + 1) * WIDTH + x]
            } else {
                0
            };

            output.push(match (top > 128, bottom > 128) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
        }

        output.push('\n');
    }

    print!("{output}");
}