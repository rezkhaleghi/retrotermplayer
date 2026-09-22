use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};

use crate::source::{VideoQuality, VideoSource};

/// Controls how FFmpeg prepares video for a renderer.
///
/// Terminal rendering is much more expensive than drawing to a real video
/// surface. These profiles intentionally keep the decoded frame small enough
/// for a terminal to redraw in real time.
#[derive(Debug, Clone, Copy)]
pub struct DecoderProfile {
    pub width: usize,
    pub height: usize,
    pub fps: u32,
}

impl DecoderProfile {
    pub const RETRO: Self = Self {
        width: 128,
        height: 72,
        fps: 15,
    };

    pub const MONO_VIDEO: Self = Self {
        width: 128,
        height: 72,
        fps: 15,
    };

    pub const VIDEO: Self = Self {
        width: 128,
        height: 72,
        fps: 15,
    };
}

/// A decoded RGB video frame.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl VideoFrame {
    pub fn pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let index = (y * self.width + x) * 3;

        (
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
        )
    }
}

/// FFmpeg-backed RGB video decoder.
///
/// FFmpeg performs the expensive media work:
///
/// source -> decode -> scale -> FPS conversion -> RGB24
///
/// The Rust side only receives the small raw RGB frames required by the
/// terminal renderer.
pub struct FfmpegDecoder {
    process: Child,
    stdout: ChildStdout,
    width: usize,
    height: usize,
    fps: u32,

    // Keep the already-resolved media URL/path so seeking does not need to
    // invoke yt-dlp again.
    input: String,
    profile: DecoderProfile,
    quality: VideoQuality,
}

impl FfmpegDecoder {
    /// Creates a decoder starting from the beginning of the source.
    pub fn new(
        source: VideoSource,
        profile: DecoderProfile,
        quality: VideoQuality,
    ) -> Result<Self, String> {
        let input = source.resolve_for_ffmpeg(quality)?;

        Self::new_from_input(input, profile, quality, 0.0)
    }

    /// Creates a decoder starting at a specific playback position.
    ///
    /// This resolves the source once. For YouTube sources, the resulting
    /// direct media URL is reused for future seeks.
    pub fn new_at_position(
        source: VideoSource,
        profile: DecoderProfile,
        quality: VideoQuality,
        position: f64,
    ) -> Result<Self, String> {
        let input = source.resolve_for_ffmpeg(quality)?;

        Self::new_from_input(input, profile, quality, position)
    }

    /// Creates a decoder from an already-resolved media input.
    pub fn new_from_input(
        input: String,
        profile: DecoderProfile,
        quality: VideoQuality,
        position: f64,
    ) -> Result<Self, String> {
        if profile.width == 0 || profile.height == 0 {
            return Err("Decoder profile dimensions must be greater than zero.".to_string());
        }

        if profile.fps == 0 {
            return Err("Decoder profile FPS must be greater than zero.".to_string());
        }

        if !position.is_finite() || position < 0.0 {
            return Err("Decoder position must be a finite non-negative value.".to_string());
        }

        let filter = format!(
            "scale={}:{}:flags=lanczos:force_original_aspect_ratio=decrease,\
             pad={}:{}:(ow-iw)/2:(oh-ih)/2,\
             eq=contrast=1.08:brightness=0.02:saturation=1.08,\
             unsharp=5:5:0.45:5:5:0,\
             fps={}",
            profile.width, profile.height, profile.width, profile.height, profile.fps
        );

        let position = position.to_string();

        let mut process = Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-loglevel",
                "quiet",
                "-ss",
                &position,
                "-i",
                &input,
                "-an",
                "-sn",
                "-vf",
                &filter,
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                format!(
                    "Could not start FFmpeg.\n\
                     Make sure FFmpeg is installed.\n\n\
                     System error: {error}"
                )
            })?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| "Could not access FFmpeg output.".to_string())?;

        Ok(Self {
            process,
            stdout,
            width: profile.width,
            height: profile.height,
            fps: profile.fps,
            input,
            profile,
            quality,
        })
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Restarts FFmpeg at a new playback position using the same resolved
    /// media input.
    pub fn seek_to(&mut self, position: f64) -> Result<(), String> {
        let decoder =
            Self::new_from_input(self.input.clone(), self.profile, self.quality, position)?;

        let old_decoder = std::mem::replace(self, decoder);

        drop(old_decoder);

        Ok(())
    }

    /// Reads exactly one RGB frame from FFmpeg.
    pub fn next_frame(&mut self, pixels: &mut Vec<u8>) -> Result<Option<VideoFrame>, String> {
        let frame_size = self.width * self.height * 3;

        if pixels.len() != frame_size {
            pixels.resize(frame_size, 0);
        }

        let mut offset = 0;

        while offset < frame_size {
            let bytes_read = self
                .stdout
                .read(&mut pixels[offset..])
                .map_err(|error| format!("Failed to read FFmpeg frame: {error}"))?;

            if bytes_read == 0 {
                let status = self
                    .process
                    .wait()
                    .map_err(|error| format!("Failed to wait for FFmpeg: {error}"))?;

                if offset == 0 && status.success() {
                    return Ok(None);
                }

                if offset > 0 {
                    return Err(format!(
                        "FFmpeg ended before a complete frame was received \
                     ({offset}/{frame_size} bytes). Exit status: {status}"
                    ));
                }

                return Err(format!("FFmpeg exited with status: {status}"));
            }

            offset += bytes_read;
        }

        Ok(Some(VideoFrame {
            width: self.width,
            height: self.height,
            pixels: std::mem::take(pixels),
        }))
    }
}

/// Returns the media duration in seconds when FFprobe can determine it.
///
/// Duration probing is intentionally separate from decoding so the player can
/// keep its playback clock independent from FFmpeg's raw-frame stream.
pub fn probe_duration(input: &str) -> Result<Option<f64>, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| {
            format!(
                "Could not start FFprobe.\n\
                 Make sure FFmpeg/FFprobe is installed.\n\n\
                 System error: {error}"
            )
        })?;

    if !output.status.success() {
        return Ok(None);
    }

    let value = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .ok();

    Ok(value.filter(|duration| duration.is_finite() && *duration >= 0.0))
}

impl Drop for FfmpegDecoder {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
