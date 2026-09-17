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
    fn render(&mut self, frame: &VideoFrame) -> String {
        let mut output = String::with_capacity(frame.width * frame.height * 10);

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            for x in 0..frame.width {
                let top = frame.pixel(x, y);

                let bottom = if y + 1 < frame.height {
                    frame.pixel(x, y + 1)
                } else {
                    (0, 0, 0)
                };

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

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r.abs_diff(g) < 8 && g.abs_diff(b) < 8 {
        if r < 8 {
            return 16;
        }

        if r > 248 {
            return 231;
        }

        return 232 + ((r as u16 - 8) * 24 / 247) as u8;
    }

    let r = ((r as u16 * 5) / 255) as u8;
    let g = ((g as u16 * 5) / 255) as u8;
    let b = ((b as u16 * 5) / 255) as u8;

    16 + 36 * r + 6 * g + b
}