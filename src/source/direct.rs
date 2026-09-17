/// Represents a directly reachable media URL.
///
/// FFmpeg is responsible for opening the URL. This can include formats
/// such as MP4, WebM, HLS streams, and other protocols supported by the
/// user's FFmpeg installation.
#[derive(Debug, Clone)]
pub struct DirectSource {
    pub url: String,
}
