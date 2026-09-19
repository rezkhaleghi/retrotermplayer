use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::source::{is_video_file, load_entries, load_entries_with_cancel};

#[derive(Debug)]
pub struct Input {
    pub source: String,
    pub renderer: usize,
}

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
             1 = ASCII Shading\n\
             2 = MonoBlock\n\
             3 = ColorBlock\n\
             4 = Video + VHS CRT\n\
             5 = TrueColor",
        )),

        (None, _) => interactive_input(),
    }
}

fn interactive_input() -> io::Result<Input> {
    loop {
        reset_terminal_for_input();
        clear_screen();

        println!("╔══════════════════════════════════════╗");
        println!("║          RETROTERMPLAYER             ║");
        println!("╚══════════════════════════════════════╝");
        println!();
        println!("1. Online");
        println!("2. Offline");
        println!("0. Exit");
        println!();

        let mode = prompt("Select: ")?;

        let source = match mode.trim() {
            "1" => match read_online_source()? {
                Some(source) => source,
                None => continue,
            },

            "2" => match read_offline_source()? {
                Some(source) => source,
                None => continue,
            },

            "0" => {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Goodbye."));
            }

            _ => {
                println!();
                println!("Please select 0, 1, or 2.");
                wait_for_enter()?;
                continue;
            }
        };

        let renderer = match read_renderer()? {
            Some(renderer) => renderer,
            None => continue,
        };

        return Ok(Input { source, renderer });
    }
}

/// Reads an online video URL.
///
/// `0` returns to the main menu.
fn read_online_source() -> io::Result<Option<String>> {
    clear_screen();

    println!("ONLINE");
    println!();
    println!("Enter 0 to go back.");
    println!();

    loop {
        let url = prompt("Video URL: ")?;
        let url = url.trim();

        if url == "0" {
            return Ok(None);
        }

        if url.is_empty() {
            println!("URL cannot be empty.");
            continue;
        }

        if url.starts_with("http://") || url.starts_with("https://") {
            return Ok(Some(url.to_string()));
        }

        println!("Please enter a valid HTTP/HTTPS URL.");
    }
}

/// Reads an offline video using path, browser, or search.
///
/// `0` returns to the main menu.
fn read_offline_source() -> io::Result<Option<String>> {
    loop {
        clear_screen();

        println!("OFFLINE");
        println!();
        println!("1. Enter path");
        println!("2. Browse");
        println!("3. Search");
        println!("0. Back");
        println!();

        let choice = prompt("Select: ")?;

        match choice.trim() {
            "0" => return Ok(None),

            "1" => {
                if let Some(path) = read_path()? {
                    return Ok(Some(path));
                }
            }

            "2" => {
                if let Some(path) = browse_directory()? {
                    return Ok(Some(path));
                }
            }

            "3" => {
                if let Some(path) = search_videos()? {
                    return Ok(Some(path));
                }
            }

            _ => {
                println!("Please select 0, 1, 2, or 3.");
                wait_for_enter()?;
            }
        }
    }
}

/// Reads a direct local video path.
///
/// `0` returns to the offline menu.
fn read_path() -> io::Result<Option<String>> {
    println!();
    println!("Enter 0 to go back.");
    println!();

    let input = prompt("Video path: ")?;
    let input = input.trim();

    if input == "0" || input.is_empty() {
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
/// `0` always means Back.
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

        println!("0. Back");

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

        let input = prompt("Select: ")?;

        if input.trim() == "0" {
            return Ok(None);
        }

        let selection = match input.trim().parse::<usize>() {
            Ok(value) if value > 0 => value,
            _ => {
                println!("Invalid selection.");
                wait_for_enter()?;
                continue;
            }
        };

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
///
/// `0` returns to the offline menu.
fn search_videos() -> io::Result<Option<String>> {
    let current = std::env::current_dir()?;

    println!();
    println!("Enter 0 to go back.");
    println!();

    let query = prompt("Search: ")?;
    let query = query.trim();

    if query.is_empty() || query == "0" {
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

    println!("0. Back");

    for (index, path) in entries.iter().enumerate() {
        println!(
            "{:>2}. {}",
            index + 1,
            path.strip_prefix(&current).unwrap_or(path).display()
        );
    }

    println!();

    loop {
        let input = prompt("Select: ")?;

        if input.trim() == "0" {
            return Ok(None);
        }

        let selection = match input.trim().parse::<usize>() {
            Ok(value) if value > 0 => value,
            _ => {
                println!("Please select a valid result.");
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
///
/// `0` returns to the main menu.
fn read_renderer() -> io::Result<Option<usize>> {
    clear_screen();

    println!("SELECT RENDER MODE");
    println!();
    println!("1. ASCII Shading");
    println!("2. MonoBlock");
    println!("3. ColorBlock");
    println!("4. Video + VHS CRT");
    println!("5. TrueColor");
    println!("0. Back");
    println!();

    loop {
        let input = prompt("Select: ")?;

        if input.trim() == "0" {
            return Ok(None);
        }

        match parse_renderer(input.trim()) {
            Ok(renderer) => return Ok(Some(renderer)),
            Err(_) => println!("Please select a renderer from 1 to 5."),
        }
    }
}

fn parse_renderer(value: &str) -> io::Result<usize> {
    let renderer = value.parse::<usize>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Renderer must be a number from 1 to 5.",
        )
    })?;

    if !(1..=5).contains(&renderer) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Renderer must be a number from 1 to 5.",
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

fn reset_terminal_for_input() {
    print!("\x1b[?1049l\x1b[?25h\x1b[?7h\x1b[0m");
    let _ = io::stdout().flush();
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

fn expand_path(input: &str) -> String {
    let input = input.trim();

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
