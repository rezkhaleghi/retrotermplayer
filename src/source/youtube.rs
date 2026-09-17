use std::process::Command;

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
    /// Keeping yt-dlp isolated here means the rest of the application
    /// doesn't need to know anything about YouTube extraction.
    pub fn resolve_stream(&self) -> Result<String, String> {
        let output = Command::new("yt-dlp")
            .args([
                "-f",
                "worstvideo",
                "-g",
                &self.url,
            ])
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

        let stream = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        if stream.is_empty() {
            return Err("yt-dlp returned an empty stream URL.".to_string());
        }

        Ok(stream)
    }
}