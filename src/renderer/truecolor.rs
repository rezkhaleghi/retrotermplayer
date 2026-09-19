use crate::decoder::VideoFrame;

use super::Renderer;

const PALETTE_SIZE: usize = 256;
const LUT_SIZE: usize = 32;
const LUT_ENTRIES: usize = LUT_SIZE * LUT_SIZE * LUT_SIZE;

/// Mode 5 video renderer.
///
/// This is intentionally different from the older ColorBlock renderer:
///
/// - RGB frames are rendered using ANSI 256-color.
/// - Each terminal cell represents two vertical pixels.
/// - The upper pixel uses the foreground color.
/// - The lower pixel uses the background color.
/// - A precomputed RGB lookup table makes palette selection cheap.
/// - Ordered dithering reduces large flat color bands.
///
/// This works on terminals that do not reliably display 24-bit ANSI
/// truecolor, including the macOS Terminal configuration this project
/// currently targets.
pub struct TrueColorRenderer {
    palette: [(u8, u8, u8); PALETTE_SIZE],
    lookup: [u8; LUT_ENTRIES],
}

impl TrueColorRenderer {
    pub fn new() -> Self {
        let palette = build_palette();
        let lookup = build_lookup(&palette);

        Self { palette, lookup }
    }

    fn color_index(&self, rgb: (u8, u8, u8)) -> u8 {
        let r = (rgb.0 as usize * (LUT_SIZE - 1)) / 255;
        let g = (rgb.1 as usize * (LUT_SIZE - 1)) / 255;
        let b = (rgb.2 as usize * (LUT_SIZE - 1)) / 255;

        let index = (r * LUT_SIZE + g) * LUT_SIZE + b;

        self.lookup[index]
    }
}

impl Default for TrueColorRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TrueColorRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        output.clear();
        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let bottom_y = (y + 1).min(frame.height - 1);

            let mut current_foreground: Option<u8> = None;
            let mut current_background: Option<u8> = None;

            for x in 0..frame.width {
                let top = dither_pixel(frame.pixel(x, y), x, y);

                let bottom = dither_pixel(frame.pixel(x, bottom_y), x, bottom_y);

                let foreground = self.color_index(top);
                let background = self.color_index(bottom);

                if current_foreground != Some(foreground) {
                    push_color(output, 38, foreground);

                    current_foreground = Some(foreground);
                }

                if current_background != Some(background) {
                    push_color(output, 48, background);

                    current_background = Some(background);
                }

                output.push('▀');
            }

            output.push_str("\x1b[0m\n");
        }
    }
}

fn push_color(output: &mut String, mode: u8, color: u8) {
    output.push_str("\x1b[");
    push_number(output, mode);
    output.push_str(";5;");
    push_number(output, color);
    output.push('m');
}

fn push_number(output: &mut String, value: u8) {
    if value >= 100 {
        output.push((b'0' + value / 100) as char);
        output.push((b'0' + (value / 10) % 10) as char);
        output.push((b'0' + value % 10) as char);
    } else if value >= 10 {
        output.push((b'0' + value / 10) as char);
        output.push((b'0' + value % 10) as char);
    } else {
        output.push((b'0' + value) as char);
    }
}

/// Builds the 256-color xterm palette.
///
/// 0..15   = ANSI colors
/// 16..231 = 6×6×6 RGB cube
/// 232..255 = grayscale ramp
fn build_palette() -> [(u8, u8, u8); PALETTE_SIZE] {
    let mut palette = [(0u8, 0u8, 0u8); PALETTE_SIZE];

    palette[..16].copy_from_slice(&[
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ]);

    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

    let mut index = 16;

    for &r in &LEVELS {
        for &g in &LEVELS {
            for &b in &LEVELS {
                palette[index] = (r, g, b);
                index += 1;
            }
        }
    }

    for i in 0..24 {
        let value = (8 + i * 10) as u8;

        palette[232 + i] = (value, value, value);
    }
    palette
}

/// Precomputes the nearest ANSI color for every 5-bit RGB combination.
///
/// 32³ = 32,768 entries, so rendering does not have to compare every
/// video pixel against all 256 palette colors.
fn build_lookup(palette: &[(u8, u8, u8); PALETTE_SIZE]) -> [u8; LUT_ENTRIES] {
    let mut lookup = [0u8; LUT_ENTRIES];

    for r in 0..LUT_SIZE {
        for g in 0..LUT_SIZE {
            for b in 0..LUT_SIZE {
                let rgb = (
                    (r * 255 / (LUT_SIZE - 1)) as u8,
                    (g * 255 / (LUT_SIZE - 1)) as u8,
                    (b * 255 / (LUT_SIZE - 1)) as u8,
                );

                let mut best_index = 0usize;
                let mut best_distance = u32::MAX;

                for (index, &color) in palette.iter().enumerate() {
                    let distance = color_distance(rgb, color);

                    if distance < best_distance {
                        best_distance = distance;
                        best_index = index;
                    }
                }

                let lookup_index = (r * LUT_SIZE + g) * LUT_SIZE + b;

                lookup[lookup_index] = best_index as u8;
            }
        }
    }

    lookup
}

/// Weighted RGB distance.
///
/// This generally produces better palette choices than treating R, G and B
/// as equally perceptually important.
fn color_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let r_mean = (a.0 as i32 + b.0 as i32) / 2;

    let r = a.0 as i32 - b.0 as i32;

    let g = a.1 as i32 - b.1 as i32;

    let blue = a.2 as i32 - b.2 as i32;

    let red_term = ((512 + r_mean) * r * r) >> 8;

    let green_term = 4 * g * g;

    let blue_term = ((767 - r_mean) * blue * blue) >> 8;

    (red_term + green_term + blue_term) as u32
}

/// Subtle ordered dithering.
///
/// The goal is to break up large areas where many RGB values collapse
/// into exactly the same ANSI-256 color without introducing visible noise.
fn dither_pixel((r, g, b): (u8, u8, u8), x: usize, y: usize) -> (u8, u8, u8) {
    const BAYER: [[i16; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

    let threshold = BAYER[y % 4][x % 4];

    let offset = threshold - 7;

    (
        dither_channel(r, offset),
        dither_channel(g, offset),
        dither_channel(b, offset),
    )
}

fn dither_channel(value: u8, offset: i16) -> u8 {
    (value as i16 + offset).clamp(0, 255) as u8
}
