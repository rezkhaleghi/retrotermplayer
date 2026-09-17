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
/// playback controls.
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
             3 = Spectrum\n\
             4 = VHS\n\
             5 = Video",
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
                 3 = Spectrum\n\
                 4 = VHS\n\
                 5 = Video",
            )
        })?
        .parse::<usize>()
        .map_err(|_| {
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

    Ok(Input { source, renderer })
}
