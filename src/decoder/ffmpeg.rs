use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};

use crate::source::VideoSource;

/// Controls how FFmpeg prepares video for a renderer.
///
/// Retro modes intentionally use a small frame size. Normal Video mode
/// uses a larger frame, but remains within a resolution that terminals can
/// realistically redraw in real time.
#[derive(Debug, Clone, Copy)]
pub struct DecoderProfile {
    pub width: usize,
    pub height: usize,
    pub fps: u32,
}

impl DecoderProfile {
    pub const RETRO: Self = Self {
        width: 100,
        height: 60,
        fps: 15,
    };

    pub const VHS: Self = Self {
        width: 110,
        height: 66,
        fps: 15,
    };

    pub const VIDEO: Self = Self {
        width: 120,
        height: 68,
        fps: 20,
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
pub struct FfmpegDecoder {
    process: Child,
    stdout: ChildStdout,
    width: usize,
    height: usize,
    fps: u32,
}

impl FfmpegDecoder {
    pub fn new(source: VideoSource, profile: DecoderProfile) -> Result<Self, String> {
        let input = source.resolve_for_ffmpeg()?;

        let filter = format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,\
             pad={}:{}:(ow-iw)/2:(oh-ih)/2,\
             fps={}",
            profile.width,
            profile.height,
            profile.width,
            profile.height,
            profile.fps
        );

        let mut process = Command::new("ffmpeg")
            .args([
                "-loglevel",
                "quiet",
                "-i",
                &input,
                "-vf",
                &filter,
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ])
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
        })
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn next_frame(&mut self) -> Result<Option<VideoFrame>, String> {
        let frame_size = self.width * self.height * 3;

        let mut pixels = vec![0u8; frame_size];
        let mut offset = 0;

        while offset < frame_size {
            let bytes_read = self
                .stdout
                .read(&mut pixels[offset..])
                .map_err(|error| {
                    format!("Failed to read FFmpeg frame: {error}")
                })?;

            if bytes_read == 0 {
                let _ = self.process.wait();
                return Ok(None);
            }

            offset += bytes_read;
        }

        Ok(Some(VideoFrame {
            width: self.width,
            height: self.height,
            pixels,
        }))
    }
}