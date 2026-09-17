use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};

use crate::source::VideoSource;

/// A decoded RGB video frame.
///
/// Renderers operate on this structure rather than dealing directly with
/// FFmpeg or raw process output. This is the main boundary between media
/// decoding and visual rendering.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl VideoFrame {
    /// Returns the RGB value of a pixel.
    pub fn pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let index = (y * self.width + x) * 3;

        (
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
        )
    }
}

/// FFmpeg-backed video decoder.
///
/// FFmpeg handles the complicated part of media playback:
/// codecs, containers, HTTP streams, local files, HLS, etc.
///
/// The decoder converts everything into a simple RGB frame stream that
/// the Rust rendering system can understand.
pub struct FfmpegDecoder {
    process: Child,
    stdout: ChildStdout,
    width: usize,
    height: usize,
}

impl FfmpegDecoder {
    pub fn new(source: VideoSource) -> Result<Self, String> {
        let input = source.resolve_for_ffmpeg()?;

        let width = 100;
        let height = 60;

        let mut process = Command::new("ffmpeg")
            .args([
                "-loglevel",
                "quiet",
                "-i",
                &input,
                "-vf",
                &format!(
                    "scale={}:{}:force_original_aspect_ratio=decrease,\
                     pad={}:{}:(ow-iw)/2:(oh-ih)/2,\
                     fps=15",
                    width, height, width, height
                ),
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
            width,
            height,
        })
    }

    /// Reads the next complete video frame.
    ///
    /// Returns None when FFmpeg reaches the end of the stream.
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