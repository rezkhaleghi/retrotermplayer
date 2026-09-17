use std::thread;
use std::time::{Duration, Instant};

use crate::{decoder::FfmpegDecoder, renderer::Renderer, terminal::Terminal};

/// Coordinates decoding, rendering, and terminal output.
///
/// The player owns one reusable output buffer. Renderers write directly into
/// that buffer instead of allocating a new String for every frame.
pub struct Player {
    decoder: FfmpegDecoder,
    renderer: Box<dyn Renderer>,
    terminal: Terminal,
    output: String,
}

impl Player {
    pub fn new(decoder: FfmpegDecoder, renderer: Box<dyn Renderer>, terminal: Terminal) -> Self {
        Self {
            decoder,
            renderer,
            terminal,

            // Start with an empty buffer. The first rendered frame allocates
            // enough memory for its output, and that allocation is then
            // reused for the remainder of playback.
            output: String::new(),
        }
    }

    pub fn play(&mut self) -> Result<(), String> {
        // FFmpeg produces frames at the FPS configured by DecoderProfile.
        let frame_duration = Duration::from_secs_f64(1.0 / self.decoder.fps() as f64);

        self.terminal.enter();

        loop {
            let frame_start = Instant::now();

            let frame = match self.decoder.next_frame()? {
                Some(frame) => frame,
                None => break,
            };

            // The renderer writes into the reusable output buffer.
            // No new String is created for this frame.
            self.renderer.render(&frame, &mut self.output);

            self.terminal.draw(&self.output);

            // Rendering and terminal output are included in the frame budget.
            // If they finish early, sleep for the remaining frame duration.
            let elapsed = frame_start.elapsed();

            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }

        Ok(())
    }
}
