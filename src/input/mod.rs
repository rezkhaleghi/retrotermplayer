use std::env;
use std::io;

/// Command-line arguments supplied to RetroTermPlayer.
#[derive(Debug)]
pub struct Input {
    pub source: String,
    pub renderer: usize,
}

/// Reads and validates command-line arguments.
///
/// Keeping all input in argv means the player never needs to read from
/// stdin during startup. This also leaves stdin available for future
/// playback controls when RetroTermPlayer is integrated into PJ-PLAYER.
pub fn read_input() -> io::Result<Input> {
    let mut args = env::args().skip(1);

    let source = args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing video URL or local file path.\n\n\
             Usage:\n\
             cargo run -- <url-or-file-path> <renderer>\n\n\
             Renderer:\n\
             1 = Retro ASCII\n\
             2 = Retro Color\n\
             3 = VHS\n\
             4 = Video",
        )
    })?;

    let renderer = args
        .next()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Missing renderer.\n\n\
                 Usage:\n\
                 cargo run -- <url-or-file-path> <renderer>\n\n\
                 Renderer:\n\
                 1 = Retro ASCII\n\
                 2 = Retro Color\n\
                 3 = VHS\n\
                 4 = Video",
            )
        })?
        .parse::<usize>()
        .map_err(|_| {
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

    Ok(Input { source, renderer })
}
