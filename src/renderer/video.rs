use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

/// Normal-quality terminal video renderer.
///
/// Terminal output is much more expensive than drawing to a real video
/// surface. This renderer therefore uses ANSI 256-color instead of truecolor
/// and represents two vertical pixels with one half-block character.
///
/// Each terminal cell represents:
///
/// ```text
/// top pixel    -> foreground color
/// bottom pixel -> background color
/// ```
///
/// This keeps the image recognizable while greatly reducing the amount of
/// data sent to the terminal.
pub struct VideoRenderer;

impl VideoRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for VideoRenderer {
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

            // Track the current ANSI colors.
            //
            // Adjacent pixels often map to the same ANSI 256-color value,
            // so there is no reason to emit another escape sequence when
            // the color has not changed.
            let mut current_top: Option<u8> = None;
            let mut current_bottom: Option<u8> = None;

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

                // Change the foreground color only when necessary.
                if current_top != Some(top_color) {
                    let _ = write!(output, "\x1b[38;5;{}m", top_color);
                    current_top = Some(top_color);
                }

                // Change the background color only when necessary.
                if current_bottom != Some(bottom_color) {
                    let _ = write!(output, "\x1b[48;5;{}m", bottom_color);
                    current_bottom = Some(bottom_color);
                }

                output.push('▀');
            }

            // Reset ANSI state at the end of each terminal row.
            output.push_str("\x1b[0m\n");
        }
    }
}

/// Converts an RGB color to the closest ANSI 256-color palette entry.
///
/// ANSI 256 provides:
///
/// - 16 basic colors
/// - 216 RGB cube colors
/// - 24 grayscale colors
///
/// The grayscale special case gives much better results for black, white,
/// and neutral parts of a video.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Nearly equal channels are better represented by the ANSI grayscale
    // palette instead of the RGB color cube.
    if r.abs_diff(g) < 8 && g.abs_diff(b) < 8 {
        if r < 8 {
            return 16;
        }

        if r > 248 {
            return 231;
        }

        return 232 + ((r as u16 - 8) * 24 / 247) as u8;
    }

    // Convert each 0..255 channel to the 0..5 ANSI color cube.
    let r = ((r as u16 * 5) / 255) as u8;
    let g = ((g as u16 * 5) / 255) as u8;
    let b = ((b as u16 * 5) / 255) as u8;

    16 + 36 * r + 6 * g + b
}
