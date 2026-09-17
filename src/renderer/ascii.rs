use crate::decoder::VideoFrame;

use super::Renderer;

/// Classic monochrome half-block renderer.
///
/// Each terminal character represents two vertical pixels:
///
///     ▀
///
/// The upper half represents the first pixel and the lower half represents
/// the second pixel. This effectively doubles the vertical resolution
/// compared with using one terminal character per pixel.
pub struct AsciiRenderer;

impl AsciiRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for AsciiRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        // Reuse the existing String allocation from the previous frame.
        // This avoids allocating a new output buffer every frame.
        output.clear();

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            // Each RGB pixel occupies three consecutive bytes.
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

                let top_brightness = brightness(top);
                let bottom_brightness = brightness(bottom);

                output.push(character(top_brightness, bottom_brightness));
            }

            output.push('\n');
        }
    }
}

/// Converts an RGB pixel into perceived brightness.
///
/// The coefficients approximate human visual sensitivity:
/// green contributes the most, followed by red, then blue.
fn brightness(pixel: (u8, u8, u8)) -> u8 {
    ((pixel.0 as u32 * 299 + pixel.1 as u32 * 587 + pixel.2 as u32 * 114) / 1000) as u8
}

/// Chooses one half-block character based on the brightness of both pixels.
fn character(top: u8, bottom: u8) -> char {
    match (top > 100, bottom > 100) {
        (true, true) => '█',
        (true, false) => '▀',
        (false, true) => '▄',
        (false, false) => ' ',
    }
}

// Keep std::fmt::Write imported explicitly because the renderer output
// implementation may later use formatted writes without changing its API.
#[allow(unused_imports)]
use std::fmt::Write as _;
