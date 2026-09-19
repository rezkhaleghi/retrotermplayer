use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

const LUT_SIZE: usize = 32;
const LUT_ENTRIES: usize = LUT_SIZE * LUT_SIZE * LUT_SIZE;

/// Normal-quality color terminal video renderer.
///
/// Each terminal cell represents two vertical pixels:
///
/// ```text
/// top pixel    -> foreground color
/// bottom pixel -> background color
/// ```
///
/// The renderer targets ANSI 256-color terminals. A perceptual palette
/// lookup table is built once during initialization so palette selection
/// remains cheap during playback.
pub struct VideoRenderer {
    palette_lut: Vec<u8>,
}

impl VideoRenderer {
    pub fn new() -> Self {
        Self {
            palette_lut: build_palette_lut(),
        }
    }
}

impl Default for VideoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for VideoRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        output.clear();
        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let top_row_start = y * frame.width * 3;

            let bottom_exists = y + 1 < frame.height;

            let bottom_row_start = if bottom_exists {
                (y + 1) * frame.width * 3
            } else {
                0
            };

            let mut current_foreground: Option<u8> = None;
            let mut current_background: Option<u8> = None;

            for x in 0..frame.width {
                let top = sample_pixel(frame, top_row_start, x);

                let bottom = if bottom_exists {
                    sample_pixel(frame, bottom_row_start, x)
                } else {
                    (0, 0, 0)
                };

                let top = enhance_pixel(frame, x, y, top, bayer_dither(x, y));

                let bottom = if bottom_exists {
                    enhance_pixel(frame, x, y + 1, bottom, bayer_dither(x, y + 1))
                } else {
                    (0, 0, 0)
                };

                let foreground = lookup_palette(&self.palette_lut, top.0, top.1, top.2);

                let background = lookup_palette(&self.palette_lut, bottom.0, bottom.1, bottom.2);

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

fn sample_pixel(frame: &VideoFrame, row_start: usize, x: usize) -> (u8, u8, u8) {
    let index = row_start + x * 3;

    (
        frame.pixels[index],
        frame.pixels[index + 1],
        frame.pixels[index + 2],
    )
}

/// Mild image enhancement performed after FFmpeg scaling.
///
/// The important part here is that we sharpen luminance rather than
/// independently sharpening RGB channels. This prevents colored halos
/// around edges.
fn enhance_pixel(
    frame: &VideoFrame,
    x: usize,
    y: usize,
    pixel: (u8, u8, u8),
    dither: i16,
) -> (u8, u8, u8) {
    let width = frame.width;
    let height = frame.height;

    let center = luminance(pixel.0, pixel.1, pixel.2);

    let left = sample_luminance(frame, x.saturating_sub(1), y);

    let right = sample_luminance(frame, (x + 1).min(width - 1), y);

    let top = sample_luminance(frame, x, y.saturating_sub(1));

    let bottom = sample_luminance(frame, x, (y + 1).min(height - 1));

    let local_average = (left + right + top + bottom) * 0.25;

    let detail = center - local_average;

    // Very restrained unsharp mask.
    let enhanced_luminance = (center + detail * 0.16).clamp(0.0, 1.0);

    let luminance_ratio = if center > 0.002 {
        enhanced_luminance / center
    } else {
        1.0
    };

    let mut r = pixel.0 as f32 * luminance_ratio;

    let mut g = pixel.1 as f32 * luminance_ratio;

    let mut b = pixel.2 as f32 * luminance_ratio;

    // Preserve shadow detail without washing out blacks.
    r = shadow_highlight_curve(r);
    g = shadow_highlight_curve(g);
    b = shadow_highlight_curve(b);

    // Very subtle contrast expansion.
    r = contrast(r, 1.025);
    g = contrast(g, 1.025);
    b = contrast(b, 1.025);

    // Ordered dithering happens before palette quantization.
    r += dither as f32;
    g += dither as f32;
    b += dither as f32;

    (
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}

/// A gentle S-curve which protects both very dark and very bright regions.
fn shadow_highlight_curve(value: f32) -> f32 {
    let normalized = (value / 255.0).clamp(0.0, 1.0);

    // Smoothstep-like curve centered around 0.5.
    let curved = normalized * normalized * (3.0 - 2.0 * normalized);

    // Keep the original image dominant.
    let result = normalized * 0.78 + curved * 0.22;

    result * 255.0
}

fn contrast(value: f32, amount: f32) -> f32 {
    ((value - 128.0) * amount + 128.0).clamp(0.0, 255.0)
}

fn sample_luminance(frame: &VideoFrame, x: usize, y: usize) -> f32 {
    let index = (y * frame.width + x) * 3;

    luminance(
        frame.pixels[index],
        frame.pixels[index + 1],
        frame.pixels[index + 2],
    )
}

fn luminance(r: u8, g: u8, b: u8) -> f32 {
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0
}

/// 4x4 Bayer ordered dithering.
///
/// The amplitude is intentionally tiny. Its job is to distribute
/// palette quantization error rather than create visible noise.
fn bayer_dither(x: usize, y: usize) -> i16 {
    const MATRIX: [[i16; 4]; 4] = [[-2, 0, -1, 1], [2, -1, 1, 0], [-1, 1, 0, 2], [1, 0, 2, -1]];

    MATRIX[y & 3][x & 3]
}

/// Converts 8-bit RGB into the 32^3 lookup table.
fn lookup_palette(lut: &[u8], r: u8, g: u8, b: u8) -> u8 {
    let r = r as usize * (LUT_SIZE - 1) / 255;
    let g = g as usize * (LUT_SIZE - 1) / 255;
    let b = b as usize * (LUT_SIZE - 1) / 255;

    let index = (r * LUT_SIZE * LUT_SIZE) + (g * LUT_SIZE) + b;

    lut[index]
}

/// Builds the complete RGB -> ANSI-256 mapping once.
///
/// 32^3 = 32,768 entries, so playback only needs three integer
/// operations and one array lookup for every pixel.
fn build_palette_lut() -> Vec<u8> {
    let mut lut = vec![0u8; LUT_ENTRIES];

    for r in 0..LUT_SIZE {
        for g in 0..LUT_SIZE {
            for b in 0..LUT_SIZE {
                let red = (r * 255 / (LUT_SIZE - 1)) as u8;

                let green = (g * 255 / (LUT_SIZE - 1)) as u8;

                let blue = (b * 255 / (LUT_SIZE - 1)) as u8;

                let index = (r * LUT_SIZE * LUT_SIZE) + (g * LUT_SIZE) + b;

                lut[index] = nearest_ansi_color(red, green, blue);
            }
        }
    }

    lut
}

/// Finds the closest ANSI-256 color.
///
/// All 256 ANSI colors are considered during initialization, not
/// during playback. This gives substantially better palette choices
/// than independently rounding RGB channels.
fn nearest_ansi_color(r: u8, g: u8, b: u8) -> u8 {
    let mut best_color = 0;
    let mut best_distance = f32::MAX;

    for color in 0u16..=255 {
        let color = color as u8;

        let (pr, pg, pb) = ansi256_rgb(color);

        let distance = perceptual_color_distance(r, g, b, pr, pg, pb);

        if distance < best_distance {
            best_distance = distance;
            best_color = color;
        }
    }

    best_color
}

/// Weighted Y/Cb/Cr-like distance.
///
/// Luminance gets the highest weight because terminal imagery is
/// especially sensitive to brightness errors. Chroma remains important
/// enough to keep saturated colors from collapsing toward gray.
fn perceptual_color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f32 {
    let y1 = 0.2126 * r1 as f32 + 0.7152 * g1 as f32 + 0.0722 * b1 as f32;

    let y2 = 0.2126 * r2 as f32 + 0.7152 * g2 as f32 + 0.0722 * b2 as f32;

    let cb1 = b1 as f32 - y1;

    let cb2 = b2 as f32 - y2;

    let cr1 = r1 as f32 - y1;

    let cr2 = r2 as f32 - y2;

    let dy = y1 - y2;

    let dcb = cb1 - cb2;

    let dcr = cr1 - cr2;

    dy * dy * 1.35 + dcb * dcb * 0.85 + dcr * dcr * 0.85
}

fn ansi256_rgb(color: u8) -> (u8, u8, u8) {
    match color {
        0..=15 => ansi_standard_rgb(color),

        16..=231 => {
            let index = color - 16;

            let r = index / 36;

            let g = (index % 36) / 6;

            let b = index % 6;

            (cube_component(r), cube_component(g), cube_component(b))
        }

        232..=255 => {
            let gray = 8 + (color - 232) * 10;

            (gray, gray, gray)
        }
    }
}

fn ansi_standard_rgb(color: u8) -> (u8, u8, u8) {
    // Standard ANSI colors.
    //
    // These values intentionally use the conventional xterm palette
    // rather than assuming that every terminal has identical RGB values.
    const COLORS: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 0, 0),
        (0, 205, 0),
        (205, 205, 0),
        (0, 0, 238),
        (205, 0, 205),
        (0, 205, 205),
        (229, 229, 229),
        (127, 127, 127),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (92, 92, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    COLORS[color as usize]
}

fn cube_component(value: u8) -> u8 {
    match value {
        0 => 0,
        1 => 95,
        2 => 135,
        3 => 175,
        4 => 215,
        _ => 255,
    }
}
