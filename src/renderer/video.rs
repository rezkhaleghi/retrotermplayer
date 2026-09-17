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
/// A small ordered dither and subtle contrast adjustment improve gradients
/// and image definition without introducing expensive image-processing
/// dependencies.
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

                let dither = bayer_dither(x, y);

                let top = adjust_pixel(top, dither);
                let bottom = adjust_pixel(bottom, dither);

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

/// Applies a small contrast adjustment and ordered dither to an RGB pixel.
///
/// The contrast adjustment is intentionally subtle. The goal is to preserve
/// the source image rather than create a stylized filter.
fn adjust_pixel(pixel: (u8, u8, u8), dither: i16) -> (u8, u8, u8) {
    (
        adjust_channel(pixel.0, dither),
        adjust_channel(pixel.1, dither),
        adjust_channel(pixel.2, dither),
    )
}

/// Adjusts one color channel.
///
/// A contrast factor of approximately 1.08 gives dark areas slightly more
/// definition while avoiding aggressive clipping of highlights.
fn adjust_channel(value: u8, dither: i16) -> u8 {
    let centered = value as i16 - 128;

    let contrast = centered * 108 / 100 + 128;
    let adjusted = contrast + dither;

    adjusted.clamp(0, 255) as u8
}

/// Returns a small ordered-dither offset.
///
/// A 2x2 Bayer matrix is enough to break up large flat color bands while
/// requiring only a few integer operations per pixel.
///
/// The result is intentionally tiny so the dither remains almost invisible.
fn bayer_dither(x: usize, y: usize) -> i16 {
    const MATRIX: [[i16; 2]; 2] = [[-2, 1], [2, -1]];

    MATRIX[y % 2][x % 2]
}

/// Converts RGB into the closest ANSI 256-color palette entry.
///
/// ANSI 256 contains a 6x6x6 RGB color cube plus a grayscale ramp. The
/// grayscale path is important for skin tones, skies, black-and-white
/// footage, and other areas where the RGB channels are close together.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Nearly equal channels are better represented by the ANSI grayscale
    // palette instead of the RGB color cube.
    if r.abs_diff(g) < 8 && g.abs_diff(b) < 8 {
        return grayscale_to_ansi(r);
    }

    // Map the color into the ANSI 6x6x6 RGB cube.
    //
    // Using rounded values instead of simple truncation gives a closer
    // approximation to the source color.
    let r_index = ((r as u16 * 5 + 127) / 255) as u8;
    let g_index = ((g as u16 * 5 + 127) / 255) as u8;
    let b_index = ((b as u16 * 5 + 127) / 255) as u8;

    16 + 36 * r_index + 6 * g_index + b_index
}

/// Maps a neutral RGB value to the ANSI grayscale ramp.
fn grayscale_to_ansi(value: u8) -> u8 {
    if value < 8 {
        return 16;
    }

    if value > 248 {
        return 231;
    }

    232 + ((value as u16 - 8) * 23 / 240) as u8
}
