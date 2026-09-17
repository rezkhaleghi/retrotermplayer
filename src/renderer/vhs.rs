use crate::decoder::VideoFrame;

use super::Renderer;

/// VHS-style renderer.
///
/// The renderer intentionally introduces visual degradation instead of
/// attempting to reproduce the source accurately. This keeps the renderer
/// independent from the video source and allows the same effect to be used
/// by other terminal applications.
pub struct VhsRenderer {
    frame_counter: u64,
}

impl VhsRenderer {
    pub fn new() -> Self {
        Self {
            frame_counter: 0,
        }
    }
}

impl Renderer for VhsRenderer {
    fn render(&mut self, frame: &VideoFrame) -> String {
        self.frame_counter += 1;

        let mut output = String::new();

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            // Horizontal scanline effect.
            let scanline = y % 4 == 0;

            for x in 0..frame.width {
                let mut top = frame.pixel(x, y);

                let mut bottom = if y + 1 < frame.height {
                    frame.pixel(x, y + 1)
                } else {
                    (0, 0, 0)
                };

                if scanline {
                    top = darken(top);
                    bottom = darken(bottom);
                }

                // Small periodic horizontal distortion.
                let shifted_x = if self.frame_counter % 37 == 0 {
                    x.saturating_sub(2)
                } else {
                    x
                };

                if shifted_x != x {
                    top = frame.pixel(shifted_x, y);

                    bottom = if y + 1 < frame.height {
                        frame.pixel(shifted_x, y + 1)
                    } else {
                        (0, 0, 0)
                    };
                }

                let top_color = rgb_to_ansi256(top.0, top.1, top.2);
                let bottom_color =
                    rgb_to_ansi256(bottom.0, bottom.1, bottom.2);

                output.push_str(&format!(
                    "\x1b[38;5;{}m\x1b[48;5;{}m▀",
                    top_color, bottom_color
                ));
            }

            output.push_str("\x1b[0m\n");
        }

        output
    }
}

fn darken(pixel: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        pixel.0 / 2,
        pixel.1 / 2,
        pixel.2 / 2,
    )
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let r = ((r as u16 * 5) / 255) as u8;
    let g = ((g as u16 * 5) / 255) as u8;
    let b = ((b as u16 * 5) / 255) as u8;

    16 + 36 * r + 6 * g + b
}