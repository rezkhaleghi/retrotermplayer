use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::source::{is_video_file, load_entries, load_entries_with_cancel};

/// Command-line arguments supplied to RetroTermPlayer.
///
/// The CLI form remains supported:
///
/// ```text
/// cargo run -- video.mkv 4
/// ```
#[derive(Debug)]
pub struct Input {
    pub source: String,
    pub renderer: usize,
}

/// Reads input.
///
/// If arguments are supplied, the traditional CLI interface is used.
/// Otherwise the interactive interface is started.
pub fn read_input() -> io::Result<Input> {
    let mut args = std::env::args().skip(1);

    match (args.next(), args.next()) {
        (Some(source), Some(renderer)) => {
            let renderer = parse_renderer(&renderer)?;

            Ok(Input { source, renderer })
        }

        (Some(_), None) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing renderer.\n\n\
             Usage:\n\
             cargo run -- <url-or-file-path> <renderer>\n\n\
             Renderer:\n\
             1 = Retro ASCII\n\
             2 = Retro Color\n\
             3 = VHS\n\
             4 = Video",
        )),

        (None, _) => interactive_input(),
    }
}

/// Interactive startup menu.
fn interactive_input() -> io::Result<Input> {
    reset_terminal_for_input();
    clear_screen();

    println!("╔══════════════════════════════════════╗");
    println!("║          RETROTERMPLAYER             ║");
    println!("╚══════════════════════════════════════╝");
    println!();
    println!("1. Online");
    println!("2. Offline");
    println!();

    let mode = prompt("Select: ")?;

    let source = match mode.trim() {
        "1" => read_online_source()?,
        "2" => read_offline_source()?,
        _ => {
            println!();
            println!("Please select 1 or 2.");
            return interactive_input();
        }
    };

    let renderer = read_renderer()?;

    Ok(Input { source, renderer })
}

/// Reads an online video URL.
fn read_online_source() -> io::Result<String> {
    println!();
    println!("ONLINE");
    println!();

    loop {
        let url = prompt("Video URL: ")?;
        let url = url.trim();

        if url.is_empty() {
            println!("URL cannot be empty.");
            continue;
        }

        if url.starts_with("http://") || url.starts_with("https://") {
            return Ok(url.to_string());
        }

        println!("Please enter a valid HTTP/HTTPS URL.");
    }
}

/// Reads an offline video using path, browser, or search.
fn read_offline_source() -> io::Result<String> {
    loop {
        println!();
        println!("OFFLINE");
        println!();
        println!("1. Enter path");
        println!("2. Browse");
        println!("3. Search");
        println!();

        let choice = prompt("Select: ")?;

        match choice.trim() {
            "1" => {
                if let Some(path) = read_path()? {
                    return Ok(path);
                }
            }

            "2" => {
                if let Some(path) = browse_directory()? {
                    return Ok(path);
                }
            }

            "3" => {
                if let Some(path) = search_videos()? {
                    return Ok(path);
                }
            }

            _ => println!("Please select 1, 2, or 3."),
        }
    }
}

/// Reads a direct local path.
fn read_path() -> io::Result<Option<String>> {
    println!();

    let input = prompt("Video path: ")?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(None);
    }

    let expanded = expand_path(input);
    let path = PathBuf::from(&expanded);

    if !path.exists() {
        println!("File does not exist.");
        return Ok(None);
    }

    if !path.is_file() {
        println!("That path is not a file.");
        return Ok(None);
    }

    if !is_video_file(&path) {
        println!("Unsupported video file.");
        return Ok(None);
    }

    Ok(Some(expanded))
}

/// Simple directory browser.
///
/// The browser starts in the current working directory.
fn browse_directory() -> io::Result<Option<String>> {
    let mut current = std::env::current_dir()?;

    loop {
        clear_screen();

        println!("OFFLINE BROWSER");
        println!();
        println!("{}", current.display());
        println!();

        let entries = match load_entries(&current, "") {
            Ok(entries) => entries,
            Err(error) => {
                println!("Could not read directory: {error}");
                wait_for_enter()?;
                return Ok(None);
            }
        };

        if entries.is_empty() {
            println!("No video files or directories found.");
            println!();
            println!("0. ..");
            println!("q. Cancel");
            println!();

            let input = prompt("Select: ")?;

            if input.trim().eq_ignore_ascii_case("q") {
                return Ok(None);
            }

            if input.trim() == "0" && !go_parent(&mut current) {
                return Ok(None);
            }

            continue;
        }

        println!("0. ..");

        for (index, path) in entries.iter().enumerate() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("?");

            if path.is_dir() {
                println!("{:>2}. {}/", index + 1, name);
            } else {
                println!("{:>2}. {}", index + 1, name);
            }
        }

        println!();
        println!("q. Cancel");
        println!();

        let input = prompt("Select: ")?;

        if input.trim().eq_ignore_ascii_case("q") {
            return Ok(None);
        }

        let selection = match input.trim().parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                println!("Invalid selection.");
                wait_for_enter()?;
                continue;
            }
        };

        if selection == 0 {
            if !go_parent(&mut current) {
                println!("Already at the filesystem root.");
                wait_for_enter()?;
            }

            continue;
        }

        let Some(path) = entries.get(selection - 1) else {
            println!("Invalid selection.");
            wait_for_enter()?;
            continue;
        };

        if path.is_dir() {
            current = path.clone();
            continue;
        }

        return Ok(Some(path.to_string_lossy().to_string()));
    }
}

/// Recursively searches from the current directory.
fn search_videos() -> io::Result<Option<String>> {
    let current = std::env::current_dir()?;

    println!();
    let query = prompt("Search: ")?;
    let query = query.trim();

    if query.is_empty() {
        return Ok(None);
    }

    println!();
    println!("Searching...");
    println!();

    let entries = load_entries_with_cancel(&current, query, None).map_err(io::Error::other)?;

    if entries.is_empty() {
        println!("No videos found.");
        wait_for_enter()?;
        return Ok(None);
    }

    for (index, path) in entries.iter().enumerate() {
        println!(
            "{:>2}. {}",
            index + 1,
            path.strip_prefix(&current).unwrap_or(path).display()
        );
    }

    println!();
    println!("q. Cancel");
    println!();

    loop {
        let input = prompt("Select: ")?;

        if input.trim().eq_ignore_ascii_case("q") {
            return Ok(None);
        }

        let selection = match input.trim().parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                println!("Invalid selection.");
                continue;
            }
        };

        let Some(path) = entries.get(selection - 1) else {
            println!("Invalid selection.");
            continue;
        };

        return Ok(Some(path.to_string_lossy().to_string()));
    }
}

/// Renderer selection shared by interactive and CLI modes.
fn read_renderer() -> io::Result<usize> {
    println!();
    println!("SELECT RENDER MODE");
    println!();
    println!("1. Retro ASCII");
    println!("2. Retro Color");
    println!("3. VHS");
    println!("4. Video");
    println!();

    loop {
        let input = prompt("Select: ")?;

        match parse_renderer(input.trim()) {
            Ok(renderer) => return Ok(renderer),
            Err(_) => println!("Please select a renderer from 1 to 4."),
        }
    }
}

fn parse_renderer(value: &str) -> io::Result<usize> {
    let renderer = value.parse::<usize>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Renderer must be a number from 1 to 4.",
        )
    })?;

    if !(1..=4).contains(&renderer) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Renderer must be a number from 1 to 4.",
        ));
    }

    Ok(renderer)
}

fn prompt(message: &str) -> io::Result<String> {
    print!("{message}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim_end_matches(&['\r', '\n'][..]).to_string())
}

fn wait_for_enter() -> io::Result<()> {
    let _ = prompt("Press Enter to continue...")?;
    Ok(())
}

/// Restores terminal output behavior before showing the interactive UI.
///
/// Playback uses the alternate screen, hides the cursor, and disables
/// automatic line wrapping. Restore those modes before printing the UI.
fn reset_terminal_for_input() {
    print!("\x1b[?1049l\x1b[?25h\x1b[?7h\x1b[0m");
    let _ = io::stdout().flush();
}

/// Clears the terminal screen.
fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

/// Expands common shell-style home-directory forms.
///
/// Interactive input does not go through the shell, so `$HOME` must be
/// expanded by the application itself.
///
/// Supported:
///
/// ```text
/// ~
/// ~/Downloads/video.mkv
/// $HOME/Downloads/video.mkv
/// "$HOME/Downloads/video.mkv"
/// "~/Downloads/video.mkv"
/// ```
fn expand_path(input: &str) -> String {
    let input = input.trim();

    // Allow paths copied with surrounding single or double quotes.
    let input = input
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            input
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(input);

    if let Ok(home) = std::env::var("HOME") {
        if input == "~" || input == "$HOME" {
            return home;
        }

        if let Some(rest) = input.strip_prefix("~/") {
            return format!("{home}/{rest}");
        }

        if let Some(rest) = input.strip_prefix("$HOME/") {
            return format!("{home}/{rest}");
        }
    }

    input.to_string()
}

fn go_parent(path: &mut PathBuf) -> bool {
    let parent = Path::new(path).parent().map(Path::to_path_buf);

    if let Some(parent) = parent {
        if parent != *path {
            *path = parent;
            return true;
        }
    }

    false
}
