use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

/// VHS-style renderer.
///
/// The renderer intentionally introduces visual degradation instead of
/// attempting to reproduce the source accurately. This keeps the renderer
/// independent from the video source and allows the same effect to be used
/// by other terminal applications.
pub struct VhsRenderer {
    /// Counts rendered frames so periodic VHS distortion can be applied.
    frame_counter: u64,
}

impl VhsRenderer {
    pub fn new() -> Self {
        Self { frame_counter: 0 }
    }
}

impl Default for VhsRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for VhsRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        self.frame_counter += 1;

        // Reuse the output allocation from the previous frame.
        output.clear();

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            // Apply a subtle dark scanline every fourth source row.
            let scanline = y % 4 == 0;

            for x in 0..frame.width {
                // VHS distortion is applied by reading a slightly shifted
                // source pixel on occasional frames.
                let shifted_x = if self.frame_counter.is_multiple_of(37) {
                    x.saturating_sub(2)
                } else {
                    x
                };

                let source_x = shifted_x;

                let top_index = (y * frame.width + source_x) * 3;

                let mut top = (
                    frame.pixels[top_index],
                    frame.pixels[top_index + 1],
                    frame.pixels[top_index + 2],
                );

                let mut bottom = if y + 1 < frame.height {
                    let bottom_index = ((y + 1) * frame.width + source_x) * 3;

                    (
                        frame.pixels[bottom_index],
                        frame.pixels[bottom_index + 1],
                        frame.pixels[bottom_index + 2],
                    )
                } else {
                    (0, 0, 0)
                };

                if scanline {
                    top = darken(top);
                    bottom = darken(bottom);
                }

                let top_color = rgb_to_ansi256(top.0, top.1, top.2);
                let bottom_color = rgb_to_ansi256(bottom.0, bottom.1, bottom.2);

                // Write directly into the reusable output buffer instead of
                // allocating a temporary String for every terminal cell.
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

/// Darkens a pixel to simulate the reduced brightness of an analog scanline.
fn darken(pixel: (u8, u8, u8)) -> (u8, u8, u8) {
    (pixel.0 / 2, pixel.1 / 2, pixel.2 / 2)
}

/// Converts RGB into the ANSI 256-color cube.
///
/// VHS intentionally uses the color cube here rather than the grayscale
/// special-case used by the normal ColorRenderer. The resulting palette
/// contributes to the intentionally degraded appearance.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let r = ((r as u16 * 5) / 255) as u8;
    let g = ((g as u16 * 5) / 255) as u8;
    let b = ((b as u16 * 5) / 255) as u8;

    16 + 36 * r + 6 * g + b
}
