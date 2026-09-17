use std::path::PathBuf;

mod direct;
mod local;
mod offline;
mod youtube;

pub use direct::DirectSource;
pub use local::LocalSource;
pub use offline::{
    is_video_file, load_entries, load_entries_with_cancel, load_video_files, SUPPORTED_EXTENSIONS,
};
pub use youtube::{VideoQuality, YouTubeSource};

#[derive(Debug, Clone)]
pub enum VideoSource {
    Local(LocalSource),
    Direct(DirectSource),
    YouTube(YouTubeSource),
}

impl VideoSource {
    pub fn description(&self) -> String {
        match self {
            Self::Local(source) => format!("Local file: {}", source.path.display()),
            Self::Direct(source) => format!("Direct URL: {}", source.url),
            Self::YouTube(source) => format!("YouTube: {}", source.url),
        }
    }

    pub fn resolve_for_ffmpeg(&self, quality: VideoQuality) -> Result<String, String> {
        match self {
            Self::Local(source) => Ok(source.path.to_string_lossy().to_string()),
            Self::Direct(source) => Ok(source.url.clone()),
            Self::YouTube(source) => source.resolve_stream(quality),
        }
    }
}

pub fn resolve_source(input: &str) -> Result<VideoSource, String> {
    let expanded = expand_home_directory(input);

    let path = PathBuf::from(&expanded);

    if path.exists() {
        if path.is_file() {
            return Ok(VideoSource::Local(LocalSource { path }));
        }

        return Err("The supplied path exists but is not a file.".to_string());
    }

    if is_youtube_url(input) {
        return Ok(VideoSource::YouTube(YouTubeSource {
            url: input.to_string(),
        }));
    }

    if is_url(input) {
        return Ok(VideoSource::Direct(DirectSource {
            url: input.to_string(),
        }));
    }

    Err("Input is neither an existing local file nor a recognized URL.".to_string())
}

fn is_youtube_url(input: &str) -> bool {
    input.contains("youtube.com/")
        || input.contains("youtu.be/")
        || input.contains("youtube-nocookie.com/")
}

fn is_url(input: &str) -> bool {
    input.starts_with("http://")
        || input.starts_with("https://")
        || input.starts_with("rtmp://")
        || input.starts_with("rtsp://")
}

fn expand_home_directory(input: &str) -> String {
    if input == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    }

    if let Some(rest) = input.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }

    input.to_string()
}
