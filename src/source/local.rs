use std::path::PathBuf;

/// Represents a video stored on the local filesystem.
///
/// The source layer does not decode the file. It only validates and
/// describes where the file is located.
#[derive(Debug, Clone)]
pub struct LocalSource {
    pub path: PathBuf,
}