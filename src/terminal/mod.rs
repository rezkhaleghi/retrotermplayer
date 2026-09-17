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
        print!("\x1b[2J\x1b[H\x1b[?25l");
        let _ = io::stdout().flush();
    }

    /// Draws a complete frame from the top-left corner.
    pub fn draw(&self, output: &str) {
        print!("\x1b[H{output}");
        let _ = io::stdout().flush();
    }

    /// Restores the terminal to a normal shell-friendly state.
    pub fn leave(&self) {
        print!("\x1b[?25h\x1b[0m\x1b[2J\x1b[H");
        let _ = io::stdout().flush();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.leave();
    }
}