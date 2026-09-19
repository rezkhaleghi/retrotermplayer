use std::io::{self, Write};

pub struct Terminal;

impl Terminal {
    pub fn new() -> Self {
        Self
    }

    pub fn enter(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Enter the alternate screen, hide the cursor, disable line wrapping,
        // clear the screen once, and move the cursor to the top-left.
        write!(stdout, "\x1b[?1049h\x1b[?25l\x1b[?7l\x1b[2J\x1b[H")?;

        stdout.flush()
    }

    pub fn draw(&self, output: &str) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Do NOT clear the terminal between frames.
        //
        // Clearing the entire screen before every frame creates a visible
        // blank interval between frames, which is especially noticeable
        // during terminal video playback.
        write!(stdout, "\x1b[H")?;

        stdout.write_all(output.as_bytes())?;
        stdout.flush()
    }

    pub fn leave(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Restore normal terminal state and leave the alternate screen.
        write!(stdout, "\x1b[0m\x1b[?7h\x1b[?25h\x1b[2J\x1b[H\x1b[?1049l")?;

        stdout.flush()
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}
