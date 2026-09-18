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
        width: 100,
        height: 60,
        fps: 15,
    };

    pub const VHS: Self = Self {
        width: 90,
        height: 54,
        fps: 15,
    };

    /// Normal terminal video mode.
    ///
    /// 90x50 provides more visual detail than the previous 80x45 profile
    /// while remaining small enough for responsive terminal rendering.
    pub const VIDEO: Self = Self {
        width: 90,
        height: 50,
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
}

impl FfmpegDecoder {
    pub fn new(
        source: VideoSource,
        profile: DecoderProfile,
        quality: VideoQuality,
    ) -> Result<Self, String> {
        let input = source.resolve_for_ffmpeg(quality)?;

        let filter = format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,\
             pad={}:{}:(ow-iw)/2:(oh-ih)/2,\
             fps={}",
            profile.width, profile.height, profile.width, profile.height, profile.fps
        );

        let mut process = Command::new("ffmpeg")
            .args([
                // FFmpeg must never read from the application's terminal.
                "-nostdin",
                // Keep FFmpeg quiet during normal playback.
                "-loglevel",
                "quiet",
                // Input video source.
                "-i",
                &input,
                // The terminal player only consumes video frames.
                "-an",
                // The terminal renderer does not process subtitles.
                "-sn",
                // Resize and convert to the renderer's target FPS.
                "-vf",
                &filter,
                // Output raw RGB frames through stdout.
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
        })
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Reads exactly one RGB frame from FFmpeg.
    ///
    /// FFmpeg writes raw video as a continuous byte stream, so a single
    /// read() is not guaranteed to return a complete frame. We therefore
    /// keep reading until the frame buffer is full.
    pub fn next_frame(&mut self) -> Result<Option<VideoFrame>, String> {
        let frame_size = self.width * self.height * 3;
        let mut pixels = vec![0u8; frame_size];
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
                    // FFmpeg reached the end of the input normally.
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
            pixels,
        }))
    }
}

impl Drop for FfmpegDecoder {
    fn drop(&mut self) {
        // std::process::Child does not terminate the OS process when the
        // Child handle is dropped. Explicitly stop FFmpeg so toggling video
        // cannot leave orphaned decoder processes behind.
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
