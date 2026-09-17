use std::thread;
use std::time::{Duration, Instant};

use crate::{
    decoder::FfmpegDecoder,
    renderer::Renderer,
    terminal::Terminal,
};

/// Coordinates the video decoder, renderer, and terminal.
///
/// The Player is intentionally unaware of where the video comes from or
/// how the video is rendered. Its only responsibility is coordinating the
/// playback loop:
///
///     decode frame → render frame → display frame
///
/// This separation allows the same Player to work with YouTube, local
/// files, direct URLs, and any renderer implementing the Renderer trait.
pub struct Player {
    decoder: FfmpegDecoder,
    renderer: Box<dyn Renderer>,
    terminal: Terminal,
}

impl Player {
    /// Creates a new video player from a decoder, renderer, and terminal.
    ///
    /// Each component has a single responsibility:
    ///
    /// - FfmpegDecoder: obtains decoded video frames.
    /// - Renderer: converts frames into terminal output.
    /// - Terminal: writes the rendered output and manages terminal state.
    pub fn new(
        decoder: FfmpegDecoder,
        renderer: Box<dyn Renderer>,
        terminal: Terminal,
    ) -> Self {
        Self {
            decoder,
            renderer,
            terminal,
        }
    }

    /// Starts the playback loop.
    ///
    /// Frames are decoded one at a time and immediately passed to the
    /// selected renderer. The player attempts to maintain the configured
    /// playback frame rate without requiring any external Rust dependency.
    pub fn play(&mut self) -> Result<(), String> {
        const FPS: u64 = 15;

        let frame_duration = Duration::from_millis(1000 / FPS);

        self.terminal.enter();

        loop {
            let frame_start = Instant::now();

            let frame = match self.decoder.next_frame()? {
                Some(frame) => frame,
                None => break,
            };

            let output = self.renderer.render(&frame);

            self.terminal.draw(&output);

            let elapsed = frame_start.elapsed();

            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }

        Ok(())
    }
}