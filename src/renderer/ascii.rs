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
    fn render(&mut self, frame: &VideoFrame) -> String {
        let mut output = String::new();

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            for x in 0..frame.width {
                let top = frame.pixel(x, y);

                let bottom = if y + 1 < frame.height {
                    frame.pixel(x, y + 1)
                } else {
                    (0, 0, 0)
                };

                let top_brightness = brightness(top);
                let bottom_brightness = brightness(bottom);

                output.push(character(top_brightness, bottom_brightness));
            }

            output.push('\n');
        }

        output
    }
}
fn brightness(pixel: (u8, u8, u8)) -> u8 {
    ((pixel.0 as u32 * 299
        + pixel.1 as u32 * 587
        + pixel.2 as u32 * 114)
        / 1000) as u8
}

fn character(top: u8, bottom: u8) -> char {
    match (top > 100, bottom > 100) {
        (true, true) => '█',
        (true, false) => '▀',
        (false, true) => '▄',
        (false, false) => ' ',
    }
}