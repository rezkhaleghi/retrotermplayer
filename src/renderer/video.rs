use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

/// Normal-quality terminal video renderer.
///
/// Each terminal cell represents two vertical pixels:
///
/// ```text
/// top pixel    -> foreground color
/// bottom pixel -> background color
/// ```
///
/// ANSI 256-color output is used instead of truecolor because terminal
/// bandwidth is an important part of the playback cost.
pub struct VideoRenderer;

impl VideoRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for VideoRenderer {
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

            let mut current_foreground: Option<u8> = None;
            let mut current_background: Option<u8> = None;

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

                let top_dither = bayer_dither(x, y);
                let bottom_dither = bayer_dither(x, y + 1);

                let top = process_pixel(top, top_dither);
                let bottom = process_pixel(bottom, bottom_dither);

                let foreground = rgb_to_ansi256(top.0, top.1, top.2);
                let background = rgb_to_ansi256(bottom.0, bottom.1, bottom.2);

                if current_foreground != Some(foreground) {
                    let _ = write!(output, "\x1b[38;5;{}m", foreground);

                    current_foreground = Some(foreground);
                }

                if current_background != Some(background) {
                    let _ = write!(output, "\x1b[48;5;{}m", background);

                    current_background = Some(background);
                }

                output.push('▀');
            }

            output.push_str("\x1b[0m\n");
        }
    }
}

fn process_pixel(pixel: (u8, u8, u8), dither: i16) -> (u8, u8, u8) {
    (
        process_channel(pixel.0, dither),
        process_channel(pixel.1, dither),
        process_channel(pixel.2, dither),
    )
}

fn process_channel(value: u8, dither: i16) -> u8 {
    let value = value as i16;

    let contrasted = ((value - 128) * 106 / 100) + 128;

    (contrasted + dither).clamp(0, 255) as u8
}

fn bayer_dither(x: usize, y: usize) -> i16 {
    const MATRIX: [[i16; 2]; 2] = [[-2, 1], [2, -1]];

    MATRIX[y & 1][x & 1]
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r.abs_diff(g) < 10 && g.abs_diff(b) < 10 {
        return grayscale_to_ansi(r);
    }

    let red = ((r as u16 * 5 + 127) / 255) as u8;
    let green = ((g as u16 * 5 + 127) / 255) as u8;
    let blue = ((b as u16 * 5 + 127) / 255) as u8;

    16 + 36 * red + 6 * green + blue
}

fn grayscale_to_ansi(value: u8) -> u8 {
    if value < 8 {
        return 16;
    }

    if value > 248 {
        return 231;
    }

    232 + ((value as u16 - 8) * 23 / 240) as u8
}
