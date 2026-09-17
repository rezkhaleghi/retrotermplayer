use crate::decoder::VideoFrame;

use super::Renderer;

/// Normal-quality terminal video renderer.
///
/// This uses ANSI truecolor and half-block characters so each terminal
/// character represents two vertical pixels. It preserves substantially
/// more color information than the Retro Color renderer.
pub struct VideoRenderer;

impl VideoRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for VideoRenderer {
    fn render(&mut self, frame: &VideoFrame) -> String {
        let mut output = String::with_capacity(frame.width * frame.height * 20);

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            for x in 0..frame.width {
                let top = frame.pixel(x, y);

                let bottom = if y + 1 < frame.height {
                    frame.pixel(x, y + 1)
                } else {
                    (0, 0, 0)
                };

                output.push_str(&format!(
                    "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                    top.0,
                    top.1,
                    top.2,
                    bottom.0,
                    bottom.1,
                    bottom.2
                ));
            }

            output.push_str("\x1b[0m\n");
        }

        output
    }
}