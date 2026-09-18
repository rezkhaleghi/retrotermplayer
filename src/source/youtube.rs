use std::process::Command;

/// Controls how much source video quality yt-dlp should request.
///
/// The renderer still receives a small frame from FFmpeg, but choosing an
/// appropriate source quality avoids downloading/processing unnecessarily
/// large YouTube video streams.
#[derive(Debug, Clone, Copy)]
pub enum VideoQuality {
    /// Lowest available video quality.
    ///
    /// Used by ASCII and Color modes where source detail is intentionally
    /// discarded by the renderer anyway.
    Low,

    /// Prefer a video stream up to 360p.
    ///
    /// Used by VHS mode where a little more source detail helps the effect
    /// without making the input unnecessarily large.
    Medium,

    /// Prefer a video stream up to 720p.
    ///
    /// Used by normal Video mode where preserving more source detail makes
    /// the terminal output look better.
    High,
}

/// Represents a YouTube video.
///
/// YouTube URLs are not passed directly to FFmpeg because YouTube usually
/// requires extraction of the underlying media stream first.
#[derive(Debug, Clone)]
pub struct YouTubeSource {
    pub url: String,
}

impl YouTubeSource {
    /// Uses yt-dlp to resolve the YouTube page into a direct media URL.
    ///
    /// The requested quality is intentionally kept here rather than leaking
    /// yt-dlp format strings into the decoder or player.
    pub fn resolve_stream(&self, quality: VideoQuality) -> Result<String, String> {
        let format = match quality {
            VideoQuality::Low => "worstvideo",
            VideoQuality::Medium => "bestvideo[height<=360]/worstvideo",
            VideoQuality::High => "bestvideo[height<=720]/bestvideo",
        };

        let output = Command::new("yt-dlp")
            .args(["-f", format, "-g", &self.url])
            .output()
            .map_err(|error| {
                format!(
                    "Could not execute yt-dlp.\n\
                     Make sure yt-dlp is installed.\n\n\
                     System error: {error}"
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(format!(
                "yt-dlp failed to resolve the YouTube video.\n\n{}",
                stderr.trim()
            ));
        }

        let stream = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if stream.is_empty() {
            return Err("yt-dlp returned an empty stream URL.".to_string());
        }

        Ok(stream)
    }
}
