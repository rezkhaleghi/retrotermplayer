
use std::fmt::Write;

use crate::decoder::VideoFrame;

use super::Renderer;

/// Normal-quality color terminal video renderer.
///
/// Each terminal cell represents two vertical pixels:
///
/// ```text
/// top pixel    -> foreground color
/// bottom pixel -> background color
/// ```
///
/// ANSI 256-color output is used because it provides much better
/// compatibility and bandwidth characteristics than truecolor.
pub struct VideoRenderer;

impl VideoRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VideoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for VideoRenderer {
    fn render(
        &mut self,
        frame: &VideoFrame,
        output: &mut String,
    ) {
        output.clear();
        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let top_row_start =
                y * frame.width * 3;

            let bottom_exists =
                y + 1 < frame.height;

            let bottom_row_start =
                if bottom_exists {
                    (y + 1) * frame.width * 3
                } else {
                    0
                };

            let mut current_foreground: Option<u8> = None;
            let mut current_background: Option<u8> = None;

            for x in 0..frame.width {
                let top_index =
                    top_row_start + x * 3;

                let top = (
                    frame.pixels[top_index],
                    frame.pixels[top_index + 1],
                    frame.pixels[top_index + 2],
                );

                let bottom = if bottom_exists {
                    let bottom_index =
                        bottom_row_start + x * 3;

                    (
                        frame.pixels[bottom_index],
                        frame.pixels[bottom_index + 1],
                        frame.pixels[bottom_index + 2],
                    )
                } else {
                    (0, 0, 0)
                };

                let top_dither =
                    bayer_dither(x, y);

                let bottom_dither =
                    bayer_dither(x, y + 1);

                let top =
                    enhance_pixel(frame, x, y, top, top_dither);

                let bottom =
                    if bottom_exists {
                        enhance_pixel(
                            frame,
                            x,
                            y + 1,
                            bottom,
                            bottom_dither,
                        )
                    } else {
                        (0, 0, 0)
                    };

                let foreground =
                    rgb_to_ansi256(
                        top.0,
                        top.1,
                        top.2,
                    );

                let background =
                    rgb_to_ansi256(
                        bottom.0,
                        bottom.1,
                        bottom.2,
                    );

                if current_foreground
                    != Some(foreground)
                {
                    let _ = write!(
                        output,
                        "\x1b[38;5;{}m",
                        foreground
                    );

                    current_foreground =
                        Some(foreground);
                }

                if current_background
                    != Some(background)
                {
                    let _ = write!(
                        output,
                        "\x1b[48;5;{}m",
                        background
                    );

                    current_background =
                        Some(background);
                }

                output.push('▀');
            }

            output.push_str("\x1b[0m\n");
        }
    }
}

/// Performs a small amount of local image enhancement.
///
/// FFmpeg already performs the primary scaling/sharpening. This pass is
/// intentionally subtle and exists mainly to recover some edge definition
/// lost when converting the RGB image into ANSI terminal colors.
fn enhance_pixel(
    frame: &VideoFrame,
    x: usize,
    y: usize,
    pixel: (u8, u8, u8),
    dither: i16,
) -> (u8, u8, u8) {
    let width = frame.width;
    let height = frame.height;

    let center_luma =
        luminance(pixel.0, pixel.1, pixel.2);

    let left =
        sample_luminance(frame, x.saturating_sub(1), y);

    let right =
        sample_luminance(
            frame,
            (x + 1).min(width - 1),
            y,
        );

    let top =
        sample_luminance(frame, x, y.saturating_sub(1));

    let bottom =
        sample_luminance(
            frame,
            x,
            (y + 1).min(height - 1),
        );

    let local_average =
        (left + right + top + bottom) * 0.25;

    let detail =
        center_luma - local_average;

    // Very subtle unsharp masking.
    //
    // Stronger sharpening looks worse at terminal resolution because
    // ANSI palette boundaries already create hard edges.
    let sharpened_luma =
        (center_luma + detail * 0.18)
            .clamp(0.0, 1.0);

    let ratio =
        if center_luma > 0.001 {
            sharpened_luma / center_luma
        } else {
            1.0
        };

    let mut r =
        (pixel.0 as f32 * ratio).clamp(0.0, 255.0);

    let mut g =
        (pixel.1 as f32 * ratio).clamp(0.0, 255.0);

    let mut b =
        (pixel.2 as f32 * ratio).clamp(0.0, 255.0);

    // Gentle contrast expansion around the midpoint.
    //
    // This is deliberately much softer than the previous 1.06 multiplier.
    r = apply_contrast(r, 1.035);
    g = apply_contrast(g, 1.035);
    b = apply_contrast(b, 1.035);

    // Tiny ordered dither before palette quantization.
    r += dither as f32;
    g += dither as f32;
    b += dither as f32;

    (
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}

fn apply_contrast(
    value: f32,
    contrast: f32,
) -> f32 {
    ((value - 128.0) * contrast + 128.0)
        .clamp(0.0, 255.0)
}

fn sample_luminance(
    frame: &VideoFrame,
    x: usize,
    y: usize,
) -> f32 {
    let index =
        (y * frame.width + x) * 3;

    let r = frame.pixels[index];
    let g = frame.pixels[index + 1];
    let b = frame.pixels[index + 2];

    luminance(r, g, b)
}

fn luminance(
    r: u8,
    g: u8,
    b: u8,
) -> f32 {
    (
        0.2126 * r as f32 +
        0.7152 * g as f32 +
        0.0722 * b as f32
    ) / 255.0
}

/// 2x2 ordered dither.
///
/// The amplitude is intentionally small because ANSI 256 already
/// introduces quantization. Larger dithering produces visible noise.
fn bayer_dither(
    x: usize,
    y: usize,
) -> i16 {
    const MATRIX: [[i16; 2]; 2] = [
        [-2, 1],
        [2, -1],
    ];

    MATRIX[y & 1][x & 1]
}

/// Converts RGB to the closest practical ANSI-256 color.
///
/// ANSI 256 contains:
///
/// - 16 standard colors
/// - 216 RGB cube colors
/// - 24 grayscale colors
///
/// We use the RGB cube plus grayscale ramp and choose based on
/// perceptual channel weighting rather than independently rounding
/// each RGB channel.
fn rgb_to_ansi256(
    r: u8,
    g: u8,
    b: u8,
) -> u8 {
    let gray_distance =
        grayscale_distance(r, g, b);

    let cube_distance =
        color_cube_distance(r, g, b);

    if gray_distance <= cube_distance {
        return grayscale_to_ansi(
            luminance_byte(r, g, b),
        );
    }

    nearest_cube_color(r, g, b)
}

fn nearest_cube_color(
    r: u8,
    g: u8,
    b: u8,
) -> u8 {
    let rf = r as f32;
    let gf = g as f32;
    let bf = b as f32;

    let r_position =
        rf / 255.0 * 5.0;

    let g_position =
        gf / 255.0 * 5.0;

    let b_position =
        bf / 255.0 * 5.0;

    let r0 =
        r_position.floor() as u8;

    let g0 =
        g_position.floor() as u8;

    let b0 =
        b_position.floor() as u8;

    let r1 =
        r0.saturating_add(1).min(5);

    let g1 =
        g0.saturating_add(1).min(5);

    let b1 =
        b0.saturating_add(1).min(5);

    let candidates = [
        (r0, g0, b0),
        (r1, g0, b0),
        (r0, g1, b0),
        (r0, g0, b1),
        (r1, g1, b0),
        (r1, g0, b1),
        (r0, g1, b1),
        (r1, g1, b1),
    ];

    let mut best_index = 0;
    let mut best_distance = f32::MAX;

    for &(cr, cg, cb) in &candidates {
        let (pr, pg, pb) =
            ansi_cube_rgb(cr, cg, cb);

        let distance =
            color_distance(
                r,
                g,
                b,
                pr,
                pg,
                pb,
            );

        if distance < best_distance {
            best_distance = distance;
            best_index =
                16 + 36 * cr + 6 * cg + cb;
        }
    }

    best_index
}

fn color_cube_distance(
    r: u8,
    g: u8,
    b: u8,
) -> f32 {
    let cube = nearest_cube_color(r, g, b);

    let (cr, cg, cb) =
        ansi256_rgb(cube);

    color_distance(
        r,
        g,
        b,
        cr,
        cg,
        cb,
    )
}

fn grayscale_distance(
    r: u8,
    g: u8,
    b: u8,
) -> f32 {
    let gray =
        luminance_byte(r, g, b);

    let ansi =
        grayscale_to_ansi(gray);

    let (gr, gg, gb) =
        ansi256_rgb(ansi);

    color_distance(
        r,
        g,
        b,
        gr,
        gg,
        gb,
    )
}

fn color_distance(
    r1: u8,
    g1: u8,
    b1: u8,
    r2: u8,
    g2: u8,
    b2: u8,
) -> f32 {
    let dr =
        r1 as f32 - r2 as f32;

    let dg =
        g1 as f32 - g2 as f32;

    let db =
        b1 as f32 - b2 as f32;

    // Green contributes more strongly to perceived brightness,
    // while blue contributes less.
    //
    // The small red/blue compensation prevents saturated colors
    // from being pulled too aggressively toward gray.
    dr * dr * 0.30 +
    dg * dg * 0.59 +
    db * db * 0.11
}

fn ansi_cube_rgb(
    r: u8,
    g: u8,
    b: u8,
) -> (u8, u8, u8) {
    (
        cube_level(r),
        cube_level(g),
        cube_level(b),
    )
}

fn cube_level(
    value: u8,
) -> u8 {
    if value == 0 {
        0
    } else {
        55 + value * 40
    }
}

fn ansi256_rgb(
    value: u8,
) -> (u8, u8, u8) {
    match value {
        16..=231 => {
            let index =
                value - 16;

            let r =
                index / 36;

            let g =
                (index % 36) / 6;

            let b =
                index % 6;

            (
                cube_component(r),
                cube_component(g),
                cube_component(b),
            )
        }

        232..=255 => {
            let gray =
                8 + (value - 232) * 10;

            (gray, gray, gray)
        }

        _ => (0, 0, 0),
    }
}

fn cube_component(
    value: u8,
) -> u8 {
    if value == 0 {
        0
    } else {
        55 + value * 40
    }
}

fn luminance_byte(
    r: u8,
    g: u8,
    b: u8,
) -> u8 {
    (
        0.2126 * r as f32 +
        0.7152 * g as f32 +
        0.0722 * b as f32
    )
    .round()
    .clamp(0.0, 255.0) as u8
}

fn grayscale_to_ansi(
    value: u8,
) -> u8 {
    if value < 8 {
        return 16;
    }

    if value > 248 {
        return 231;
    }

    232 + (
        (value as u16 - 8) * 23 / 240
    ) as u8
}

