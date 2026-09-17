use std::thread;
use std::time::{Duration, Instant};

use crate::{
    decoder::FfmpegDecoder,
    renderer::Renderer,
    terminal::Terminal,
};

/// Coordinates decoding, rendering, and terminal output.
///
/// Playback timing is based on the decoder's configured FPS rather than
/// using an arbitrary sleep. This keeps each visual mode synchronized with
/// the frame rate produced by FFmpeg.
pub struct Player {
    decoder: FfmpegDecoder,
    renderer: Box<dyn Renderer>,
    terminal: Terminal,
}

impl Player {
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

    pub fn play(&mut self) -> Result<(), String> {
        let frame_duration =
            Duration::from_secs_f64(1.0 / self.decoder.fps() as f64);

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