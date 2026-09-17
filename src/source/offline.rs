use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Video formats that can be selected from the offline browser.
///
/// FFmpeg supports many more formats, but these cover the common local
/// video files without turning the browser into a format registry.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "webm", "mov", "m4v", "avi", "wmv", "flv", "mpg", "mpeg", "ts", "mts", "m2ts",
    "3gp", "ogv",
];

/// Loads video files directly inside a folder.
pub fn load_video_files(folder: &Path) -> Result<Vec<PathBuf>, String> {
    if !folder.is_dir() {
        return Err(format!("Not a folder: {}", folder.display()));
    }

    let mut files: Vec<PathBuf> = std::fs::read_dir(folder)
        .map_err(|error| format!("Could not read folder: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| !is_hidden(path) && is_video_file(path))
        .collect();

    files.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));

    Ok(files)
}

/// Loads directories and video files from a folder.
///
/// Directories are returned first, followed by video files.
pub fn load_entries(folder: &Path, query: &str) -> Result<Vec<PathBuf>, String> {
    load_entries_with_cancel(folder, query, None)
}

/// Same as `load_entries`, but allows a recursive search to be cancelled.
pub fn load_entries_with_cancel(
    folder: &Path,
    query: &str,
    cancel: Option<&AtomicBool>,
) -> Result<Vec<PathBuf>, String> {
    if !folder.is_dir() {
        return Err(format!("Not a folder: {}", folder.display()));
    }

    let query = query.to_lowercase();

    if !query.is_empty() {
        return load_matching_video_files(folder, &query, cancel);
    }

    let mut entries: Vec<PathBuf> = std::fs::read_dir(folder)
        .map_err(|error| format!("Could not read folder: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            if is_hidden(path) {
                return false;
            }

            path.is_dir() || is_video_file(path)
        })
        .collect();

    entries.sort_by_key(|path| {
        (
            !path.is_dir(),
            path.file_name().map(|name| name.to_os_string()),
        )
    });

    Ok(entries)
}

/// Recursively searches for video files whose full path contains the query.
fn load_matching_video_files(
    folder: &Path,
    query: &str,
    cancel: Option<&AtomicBool>,
) -> Result<Vec<PathBuf>, String> {
    let mut entries = Vec::new();

    let read_dir = match std::fs::read_dir(folder) {
        Ok(read_dir) => read_dir,
        Err(_) => return Ok(entries),
    };

    for entry in read_dir.filter_map(Result::ok) {
        if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Ok(entries);
        }

        let path = entry.path();

        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if is_hidden(&path) || file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            entries.extend(load_matching_video_files(&path, query, cancel)?);
        } else if file_type.is_file()
            && is_video_file(&path)
            && path.to_string_lossy().to_lowercase().contains(query)
        {
            entries.push(path);
        }
    }

    entries.sort_by_key(|path| path.to_string_lossy().to_lowercase());

    Ok(entries)
}

/// Returns true when the path has a supported video extension.
pub fn is_video_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                SUPPORTED_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
            })
}

/// Hidden files and directories are ignored by the browser.
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}
