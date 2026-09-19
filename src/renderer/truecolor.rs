use crate::decoder::VideoFrame;

use super::Renderer;

const PALETTE_SIZE: usize = 256;
const LUT_SIZE: usize = 32;
const LUT_ENTRIES: usize = LUT_SIZE * LUT_SIZE * LUT_SIZE;

const CONTRAST: f32 = 1.06;
const SATURATION: f32 = 1.10;
const BRIGHTNESS: f32 = 1.02;

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
                let top = smooth_pixel(frame, x, y);
                let bottom = smooth_pixel(frame, x, bottom_y);

                let top = enhance_pixel(top);
                let bottom = enhance_pixel(bottom);

                let top = dither_pixel(top, x, y);
                let bottom = dither_pixel(bottom, x, bottom_y);

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

fn smooth_pixel(
    frame: &VideoFrame,
    x: usize,
    y: usize,
) -> (u8, u8, u8) {
    let x1 = x.saturating_sub(1);
    let x2 = (x + 1).min(frame.width - 1);

    let y1 = y.saturating_sub(1);
    let y2 = (y + 1).min(frame.height - 1);

    let center = frame.pixel(x, y);
    let left = frame.pixel(x1, y);
    let right = frame.pixel(x2, y);
    let above = frame.pixel(x, y1);
    let below = frame.pixel(x, y2);

    // Keep the center pixel dominant.
    //
    // This removes isolated noisy ANSI colors without blurring
    // the image excessively.
    (
        weighted_average(
            center.0,
            left.0,
            right.0,
            above.0,
            below.0,
        ),
        weighted_average(
            center.1,
            left.1,
            right.1,
            above.1,
            below.1,
        ),
        weighted_average(
            center.2,
            left.2,
            right.2,
            above.2,
            below.2,
        ),
    )
}

fn weighted_average(
    center: u8,
    left: u8,
    right: u8,
    above: u8,
    below: u8,
) -> u8 {
    let value =
        center as u16 * 4
        + left as u16
        + right as u16
        + above as u16
        + below as u16;

    (value / 8) as u8
}

fn enhance_pixel(
    (r, g, b): (u8, u8, u8),
) -> (u8, u8, u8) {
    let mut r = r as f32 / 255.0;
    let mut g = g as f32 / 255.0;
    let mut b = b as f32 / 255.0;

    let luminance =
        0.2126 * r
        + 0.7152 * g
        + 0.0722 * b;

    // Restore color after downsampling.
    r = luminance + (r - luminance) * SATURATION;
    g = luminance + (g - luminance) * SATURATION;
    b = luminance + (b - luminance) * SATURATION;

    // Improve separation between dark / mid / bright areas.
    r = (r - 0.5) * CONTRAST + 0.5;
    g = (g - 0.5) * CONTRAST + 0.5;
    b = (b - 0.5) * CONTRAST + 0.5;

    r *= BRIGHTNESS;
    g *= BRIGHTNESS;
    b *= BRIGHTNESS;

    (
        to_byte(r),
        to_byte(g),
        to_byte(b),
    )
}

fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0) as u8
}

fn dither_pixel(
    (r, g, b): (u8, u8, u8),
    x: usize,
    y: usize,
) -> (u8, u8, u8) {
    const BAYER: [[i16; 4]; 4] = [
        [0, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5],
    ];

    let offset =
        BAYER[y % 4][x % 4] - 7;

    (
        dither_channel(r, offset),
        dither_channel(g, offset),
        dither_channel(b, offset),
    )
}

fn dither_channel(
    value: u8,
    offset: i16,
) -> u8 {
    (value as i16 + offset)
        .clamp(0, 255) as u8
}

fn push_color(
    output: &mut String,
    mode: u8,
    color: u8,
) {
    output.push_str("\x1b[");
    push_number(output, mode);
    output.push_str(";5;");
    push_number(output, color);
    output.push('m');
}

fn push_number(
    output: &mut String,
    value: u8,
) {
    if value >= 100 {
        output.push(
            (b'0' + value / 100) as char,
        );
        output.push(
            (b'0' + (value / 10) % 10) as char,
        );
        output.push(
            (b'0' + value % 10) as char,
        );
    } else if value >= 10 {
        output.push(
            (b'0' + value / 10) as char,
        );
        output.push(
            (b'0' + value % 10) as char,
        );
    } else {
        output.push(
            (b'0' + value) as char,
        );
    }
}

fn build_palette() -> [(u8, u8, u8); PALETTE_SIZE] {
    let mut palette =
        [(0u8, 0u8, 0u8); PALETTE_SIZE];

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

    const LEVELS: [u8; 6] =
        [0, 95, 135, 175, 215, 255];

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

        palette[232 + i] =
            (value, value, value);
    }

    palette
}

fn build_lookup(
    palette: &[(u8, u8, u8); PALETTE_SIZE],
) -> [u8; LUT_ENTRIES] {
    let mut lookup =
        [0u8; LUT_ENTRIES];

    for r in 0..LUT_SIZE {
        for g in 0..LUT_SIZE {
            for b in 0..LUT_SIZE {
                let rgb = (
                    (r * 255 / (LUT_SIZE - 1)) as u8,
                    (g * 255 / (LUT_SIZE - 1)) as u8,
                    (b * 255 / (LUT_SIZE - 1)) as u8,
                );

                let mut best = 0usize;
                let mut best_distance = u32::MAX;

                for (index, &color)
                    in palette.iter().enumerate()
                {
                    let distance =
                        color_distance(rgb, color);

                    if distance < best_distance {
                        best_distance = distance;
                        best = index;
                    }
                }

                let lookup_index =
                    (r * LUT_SIZE + g)
                        * LUT_SIZE
                        + b;

                lookup[lookup_index] =
                    best as u8;
            }
        }
    }

    lookup
}

fn color_distance(
    a: (u8, u8, u8),
    b: (u8, u8, u8),
) -> u32 {
    let ar = a.0 as i64;
    let ag = a.1 as i64;
    let ab = a.2 as i64;

    let br = b.0 as i64;
    let bg = b.1 as i64;
    let bb = b.2 as i64;

    let r_mean =
        (ar + br) / 2;

    let dr = ar - br;
    let dg = ag - bg;
    let db = ab - bb;

    let red =
        ((512 + r_mean) * dr * dr) / 256;

    let green =
        4 * dg * dg;

    let blue =
        ((767 - r_mean) * db * db) / 256;

    (red + green + blue) as u32
}