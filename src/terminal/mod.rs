use std::io::{self, Write};

pub struct Terminal;

impl Terminal {
    pub fn new() -> Self {
        Self
    }

    pub fn enter(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        write!(stdout, "\x1b[?1049h\x1b[?25l\x1b[?7l\x1b[2J\x1b[H")?;

        stdout.flush()
    }

    pub fn draw(&self, output: &str) -> io::Result<()> {
        let mut stdout = io::stdout();

        write!(stdout, "\x1b[H\x1b[2J")?;
        stdout.write_all(output.as_bytes())?;
        stdout.flush()
    }

    pub fn leave(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        write!(stdout, "\x1b[0m\x1b[?7h\x1b[?25h\x1b[2J\x1b[H\x1b[?1049l")?;

        stdout.flush()
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}
