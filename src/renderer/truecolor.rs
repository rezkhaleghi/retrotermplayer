use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

/// 24-bit truecolor half-block renderer.
///
/// Each terminal cell represents two vertical RGB pixels:
///
///     upper pixel -> foreground
///     lower pixel -> background
///
/// Unlike the ColorBlock and Video renderers, this does not quantize the
/// image into the ANSI 256-color palette.
pub struct TrueColorRenderer;

impl TrueColorRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TrueColorRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TrueColorRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        output.clear();
        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let top_row_start = y * frame.width * 3;

            let bottom_row_start = if y + 1 < frame.height {
                (y + 1) * frame.width * 3
            } else {
                0
            };

            let mut current_foreground: Option<(u8, u8, u8)> = None;
            let mut current_background: Option<(u8, u8, u8)> = None;

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

                if current_foreground != Some(top) {
                    let _ = write!(output, "\x1b[38;2;{};{};{}m", top.0, top.1, top.2);

                    current_foreground = Some(top);
                }

                if current_background != Some(bottom) {
                    let _ = write!(output, "\x1b[48;2;{};{};{}m", bottom.0, bottom.1, bottom.2);

                    current_background = Some(bottom);
                }

                output.push('▀');
            }

            output.push_str("\x1b[0m\n");
        }
    }
}
