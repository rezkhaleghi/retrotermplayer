use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

/// 256-color ANSI half-block renderer.
///
/// The upper half of `▀` uses the foreground color while the lower half
/// uses the background color. This allows two RGB pixels to be represented
/// by a single terminal character.
pub struct ColorRenderer;

impl ColorRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for ColorRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        // Reuse the same output allocation for every frame.
        output.clear();

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let top_row_start = y * frame.width * 3;

            let bottom_row_start = if y + 1 < frame.height {
                (y + 1) * frame.width * 3
            } else {
                0
            };

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

                let top_color = rgb_to_ansi256(top.0, top.1, top.2);
                let bottom_color = rgb_to_ansi256(bottom.0, bottom.1, bottom.2);

                // write! writes directly into the reusable String.
                // Unlike format!, this does not create a temporary String.
                let _ = write!(
                    output,
                    "\x1b[38;5;{}m\x1b[48;5;{}m▀",
                    top_color, bottom_color
                );
            }

            output.push_str("\x1b[0m\n");
        }
    }
}

/// Converts RGB into the closest color in the ANSI 256-color palette.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Treat nearly-equal RGB values as grayscale. This produces a better
    // mapping for neutral colors than forcing them into the color cube.
    if r.abs_diff(g) < 8 && g.abs_diff(b) < 8 {
        if r < 8 {
            return 16;
        }

        if r > 248 {
            return 231;
        }

        return 232 + ((r as u16 - 8) * 24 / 247) as u8;
    }

    // Map each 0..255 RGB channel into the 0..5 ANSI color cube.
    let r = ((r as u16 * 5) / 255) as u8;
    let g = ((g as u16 * 5) / 255) as u8;
    let b = ((b as u16 * 5) / 255) as u8;

    16 + 36 * r + 6 * g + b
}
