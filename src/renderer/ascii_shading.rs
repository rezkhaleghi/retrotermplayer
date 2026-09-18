use crate::decoder::VideoFrame;

use super::Renderer;

/// Monochrome ASCII renderer using character density for shading.
///
/// Dark pixels become dense characters such as `@` and `#`.
/// Bright pixels become lighter characters such as `.` or a space.
pub struct AsciiShadingRenderer;

impl AsciiShadingRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AsciiShadingRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for AsciiShadingRenderer {
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

                let brightness = ((brightness(top) as u16 + brightness(bottom) as u16) / 2) as u8;

                output.push(character(brightness));
            }

            output.push('\n');
        }
    }
}

/// Converts an RGB pixel into perceived brightness.
fn brightness(pixel: (u8, u8, u8)) -> u8 {
    ((pixel.0 as u32 * 299 + pixel.1 as u32 * 587 + pixel.2 as u32 * 114) / 1000) as u8
}

/// Maps brightness to ASCII character density.
///
/// Dark → light:
///
/// @ # 8 & o : , . space
fn character(brightness: u8) -> char {
    const RAMP: &[u8] = b"@#8&o:,. ";

    let index = (brightness as usize * (RAMP.len() - 1)) / 255;

    RAMP[index] as char
}
