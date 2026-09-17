use std::io::{self, Write};

pub struct Terminal;

impl Terminal {
    pub fn new() -> Self {
        Self
    }

    pub fn enter(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Enter alternate screen, hide cursor, clear screen and reset
        // scrolling/wrapping state.
        let _ = handle.write_all(
            b"\x1b[?1049h\
              \x1b[?25l\
              \x1b[2J\
              \x1b[H\
              \x1b[?7l",
        );

        let _ = handle.flush();
    }

    pub fn draw(&self, output: &str) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Always:
        //   1. move to the top-left
        //   2. clear the previous frame
        //   3. draw the new frame
        //
        // This prevents old/wrapped content from surviving a resize.
        let _ = handle.write_all(b"\x1b[H\x1b[2J");
        let _ = handle.write_all(output.as_bytes());
        let _ = handle.flush();
    }

    pub fn leave(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Restore normal terminal behaviour.
        let _ = handle.write_all(
            b"\x1b[?7h\
              \x1b[?25h\
              \x1b[0m\
              \x1b[2J\
              \x1b[H\
              \x1b[?1049l",
        );

        let _ = handle.flush();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.leave();
    }
}
