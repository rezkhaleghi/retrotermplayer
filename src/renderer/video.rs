use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

/// Normal-quality terminal video renderer.
///
/// Uses ANSI truecolor and half-block characters. Each terminal cell
/// represents two vertical pixels, giving substantially better detail
/// than one-character-per-pixel rendering while keeping the output
/// manageable for a normal terminal.
pub struct VideoRenderer;

impl VideoRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for VideoRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        // Reuse the same String allocation for every frame.
        output.clear();

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let top_row_start = y * frame.width * 3;

            let bottom_row_start = if y + 1 < frame.height {
                (y + 1) * frame.width * 3
            } else {
                0
            };

            // ANSI state is cached for each row. If adjacent pixels share
            // the same color, we don't emit another escape sequence.
            let mut current_top: Option<(u8, u8, u8)> = None;
            let mut current_bottom: Option<(u8, u8, u8)> = None;

            for x in 0..frame.width {
                let top_index = top_row_start + x * 3;

                let top = (
                    frame.pixels[top_index],
                    frame.pixels[top_index + 1],
                    frame.pixels[top_index + 2],
                );

                let bottom = if y + 1 < frame.height {
                    let bottom_index = bottom_row_start + x * 3;

                    (
                        frame.pixels[bottom_index],
                        frame.pixels[bottom_index + 1],
                        frame.pixels[bottom_index + 2],
                    )
                } else {
                    (0, 0, 0)
                };

                // Only emit a foreground color escape when the color changed.
                if current_top != Some(top) {
                    let _ = write!(output, "\x1b[38;2;{};{};{}m", top.0, top.1, top.2);

                    current_top = Some(top);
                }

                // Only emit a background color escape when the color changed.
                if current_bottom != Some(bottom) {
                    let _ = write!(output, "\x1b[48;2;{};{};{}m", bottom.0, bottom.1, bottom.2);

                    current_bottom = Some(bottom);
                }

                output.push('▀');
            }

            // Reset ANSI state at the end of each terminal row.
            output.push_str("\x1b[0m\n");
        }
    }
}
