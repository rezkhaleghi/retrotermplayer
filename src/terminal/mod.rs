use std::io::{self, Write};

/// Owns the terminal state used during playback.
///
/// The terminal is treated as a resource: when playback starts we modify
/// its visual state, and when this object is dropped we restore that state.
/// This prevents one execution of the application from affecting the next
/// execution in the same shell.
pub struct Terminal;

impl Terminal {
    pub fn new() -> Self {
        Self
    }

    /// Enters the playback display state.
    ///
    /// We only modify visual terminal state here. We deliberately do not
    /// enable raw mode or alter keyboard/input modes, because input is
    /// currently handled by standard stdin.
    pub fn enter(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Clear the terminal, move the cursor to the top-left corner, and
        // hide the cursor while video is being rendered.
        let _ = handle.write_all(b"\x1b[2J\x1b[H\x1b[?25l");

        let _ = handle.flush();
    }

    /// Draws a complete frame from the top-left corner.
    ///
    /// write_all avoids the formatting machinery used by print! and writes
    /// the already-rendered ANSI buffer directly to stdout.
    pub fn draw(&self, output: &str) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let _ = handle.write_all(b"\x1b[H");
        let _ = handle.write_all(output.as_bytes());

        // Flush once per complete frame so the terminal receives the whole
        // frame together rather than buffering it indefinitely.
        let _ = handle.flush();
    }

    /// Restores the terminal to a normal shell-friendly state.
    pub fn leave(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        // Show the cursor, reset ANSI state, clear the screen, and return
        // the cursor to the top-left corner.
        let _ = handle.write_all(b"\x1b[?25h\x1b[0m\x1b[2J\x1b[H");

        let _ = handle.flush();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.leave();
    }
}
