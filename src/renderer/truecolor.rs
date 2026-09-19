use crate::decoder::VideoFrame;

use super::Renderer;

const PALETTE_SIZE: usize = 256;

/*
 * Image tuning.
 *
 * Luminance is kept relatively sharp because that is where most of
 * the perceived detail comes from at 100x60.
 *
 * Chroma is deliberately smoother because low-resolution chroma
 * produces the red/yellow/blue speckling we were seeing.
 */
const CONTRAST: f32 = 1.08;
const BRIGHTNESS: f32 = 1.015;
const CHROMA_STRENGTH: f32 = 0.90;

/*
 * Minimum chroma strength required before we allow a strongly
 * saturated ANSI color.
 *
 * This prevents tiny compression artifacts from becoming red/yellow
 * pixels while still allowing real colors through.
 */
const CHROMA_THRESHOLD: f32 = 0.075;

/*
 * How much neighboring chroma contributes.
 *
 * Luminance does NOT receive this smoothing.
 */
const CHROMA_NEIGHBOR_WEIGHT: f32 = 0.28;

pub struct TrueColorRenderer {
    palette: [(u8, u8, u8); PALETTE_SIZE],
}

impl TrueColorRenderer {
    pub fn new() -> Self {
        Self {
            palette: build_palette(),
        }
    }

    fn color_index(
        &self,
        rgb: (u8, u8, u8),
    ) -> u8 {
        best_palette_index(
            rgb,
            &self.palette,
        )
    }
}

impl Default for TrueColorRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TrueColorRenderer {
    fn render(
        &mut self,
        frame: &VideoFrame,
        output: &mut String,
    ) {
        output.clear();
        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let bottom_y =
                (y + 1).min(frame.height - 1);

            let mut current_fg: Option<u8> = None;
            let mut current_bg: Option<u8> = None;

            for x in 0..frame.width {
                /*
                 * Upper half of the terminal character.
                 */
                let top =
                    prepare_pixel(frame, x, y);

                /*
                 * Lower half.
                 */
                let bottom =
                    prepare_pixel(frame, x, bottom_y);

                let fg =
                    self.color_index(top);

                let bg =
                    self.color_index(bottom);

                if current_fg != Some(fg) {
                    push_color(
                        output,
                        38,
                        fg,
                    );

                    current_fg = Some(fg);
                }

                if current_bg != Some(bg) {
                    push_color(
                        output,
                        48,
                        bg,
                    );

                    current_bg = Some(bg);
                }

                output.push('▀');
            }

            output.push_str("\x1b[0m\n");
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Pixel processing                                                           */
/* -------------------------------------------------------------------------- */

fn prepare_pixel(
    frame: &VideoFrame,
    x: usize,
    y: usize,
) -> (u8, u8, u8) {
    /*
     * Center pixel provides detail.
     */
    let center =
        rgb_to_ycbcr(frame.pixel(x, y));

    /*
     * Chroma is averaged independently from luminance.
     *
     * This is the most important difference from the previous
     * RGB smoothing approach.
     */
    let chroma =
        smooth_chroma(frame, x, y);

    /*
     * Keep center luminance. This prevents faces and edges from
     * becoming blurry.
     */
    let y_value =
        enhance_luminance(center.0);

    /*
     * Combine sharp luminance with stable chroma.
     */
    ycbcr_to_rgb((
        y_value,
        chroma.0,
        chroma.1,
    ))
}

/*
 * YCbCr conversion.
 *
 * Y  = brightness/detail
 * Cb = blue chroma
 * Cr = red chroma
 */
fn rgb_to_ycbcr(
    rgb: (u8, u8, u8),
) -> (f32, f32, f32) {
    let r = rgb.0 as f32 / 255.0;
    let g = rgb.1 as f32 / 255.0;
    let b = rgb.2 as f32 / 255.0;

    let y =
        0.299 * r
        + 0.587 * g
        + 0.114 * b;

    let cb =
        -0.168736 * r
        -0.331264 * g
        + 0.5 * b;

    let cr =
        0.5 * r
        -0.418688 * g
        -0.081312 * b;

    (y, cb, cr)
}

fn ycbcr_to_rgb(
    ycbcr: (f32, f32, f32),
) -> (u8, u8, u8) {
    let y = ycbcr.0;
    let cb = ycbcr.1;
    let cr = ycbcr.2;

    let r =
        y + 1.402 * cr;

    let g =
        y
        - 0.344136 * cb
        - 0.714136 * cr;

    let b =
        y + 1.772 * cb;

    (
        to_byte(r),
        to_byte(g),
        to_byte(b),
    )
}

/*
 * Spatially smooth only chroma.
 *
 * The center gets most of the weight.
 * The surrounding pixels stabilize color transitions.
 */
fn smooth_chroma(
    frame: &VideoFrame,
    x: usize,
    y: usize,
) -> (f32, f32) {
    let x_left =
        x.saturating_sub(1);

    let x_right =
        (x + 1).min(frame.width - 1);

    let y_top =
        y.saturating_sub(1);

    let y_bottom =
        (y + 1).min(frame.height - 1);

    let center =
        rgb_to_ycbcr(frame.pixel(x, y));

    let left =
        rgb_to_ycbcr(frame.pixel(x_left, y));

    let right =
        rgb_to_ycbcr(frame.pixel(x_right, y));

    let top =
        rgb_to_ycbcr(frame.pixel(x, y_top));

    let bottom =
        rgb_to_ycbcr(frame.pixel(x, y_bottom));

    let neighbor_cb =
        (left.1
            + right.1
            + top.1
            + bottom.1)
            / 4.0;

    let neighbor_cr =
        (left.2
            + right.2
            + top.2
            + bottom.2)
            / 4.0;

    /*
     * Blend center chroma with neighborhood chroma.
     */
    let cb =
        center.1
            * (1.0 - CHROMA_NEIGHBOR_WEIGHT)
        + neighbor_cb
            * CHROMA_NEIGHBOR_WEIGHT;

    let cr =
        center.2
            * (1.0 - CHROMA_NEIGHBOR_WEIGHT)
        + neighbor_cr
            * CHROMA_NEIGHBOR_WEIGHT;

    /*
     * Suppress very weak chroma.
     */
    let cb =
        suppress_weak_chroma(cb);

    let cr =
        suppress_weak_chroma(cr);

    (
        cb * CHROMA_STRENGTH,
        cr * CHROMA_STRENGTH,
    )
}

fn suppress_weak_chroma(
    value: f32,
) -> f32 {
    let magnitude =
        value.abs();

    if magnitude < CHROMA_THRESHOLD {
        /*
         * Smoothly move weak chroma toward zero.
         * This avoids a hard color/no-color boundary.
         */
        let ratio =
            magnitude / CHROMA_THRESHOLD;

        value * ratio * ratio
    } else {
        value
    }
}

/*
 * Sharpen contrast without touching chroma.
 */
fn enhance_luminance(
    value: f32,
) -> f32 {
    let value =
        (value - 0.5)
            * CONTRAST
            + 0.5;

    (value * BRIGHTNESS)
        .clamp(0.0, 1.0)
}

/* -------------------------------------------------------------------------- */
/* ANSI palette                                                               */
/* -------------------------------------------------------------------------- */

fn build_palette() -> [(u8, u8, u8); PALETTE_SIZE] {
    let mut palette =
        [(0u8, 0u8, 0u8); PALETTE_SIZE];

    /*
     * Standard ANSI colors.
     */
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

    /*
     * ANSI 216-color cube.
     */
    const LEVELS: [u8; 6] =
        [0, 95, 135, 175, 215, 255];

    let mut index = 16;

    for &r in &LEVELS {
        for &g in &LEVELS {
            for &b in &LEVELS {
                palette[index] =
                    (r, g, b);

                index += 1;
            }
        }
    }

    /*
     * ANSI grayscale ramp.
     */
    for i in 0..24 {
        let value =
            (8 + i * 10) as u8;

        palette[232 + i] =
            (value, value, value);
    }

    palette
}

/* -------------------------------------------------------------------------- */
/* Palette selection                                                         */
/* -------------------------------------------------------------------------- */

fn best_palette_index(
    rgb: (u8, u8, u8),
    palette: &[(u8, u8, u8); PALETTE_SIZE],
) -> u8 {
    let source =
        rgb_to_ycbcr(rgb);

    let source_chroma =
        chroma_strength(
            source.1,
            source.2,
        );

    let mut best =
        0usize;

    let mut best_distance =
        f32::MAX;

    for (index, &candidate) in
        palette.iter().enumerate()
    {
        let candidate_ycbcr =
            rgb_to_ycbcr(candidate);

        let candidate_chroma =
            chroma_strength(
                candidate_ycbcr.1,
                candidate_ycbcr.2,
            );

        /*
         * Luminance is more important than chroma.
         *
         * At 100x60, preserving the shape of the face/body/etc.
         * matters more than matching a tiny color difference.
         */
        let dy =
            source.0
                - candidate_ycbcr.0;

        let dcb =
            source.1
                - candidate_ycbcr.1;

        let dcr =
            source.2
                - candidate_ycbcr.2;

        let mut distance =
            dy * dy * 5.0
            + dcb * dcb * 1.7
            + dcr * dcr * 1.7;

        /*
         * Neutral source pixels should strongly prefer neutral
         * palette colors.
         */
        if source_chroma < 0.055
            && candidate_chroma > 0.10
        {
            distance += 0.20;
        }

        /*
         * Mildly colored pixels should not jump to extremely
         * saturated ANSI colors.
         */
        if source_chroma >= 0.055
            && source_chroma < 0.13
            && candidate_chroma > 0.30
        {
            distance += 0.045;
        }

        /*
         * Clearly colorful source pixels should retain color.
         */
        if source_chroma > 0.15
            && candidate_chroma < 0.035
        {
            distance += 0.035;
        }

        if distance < best_distance {
            best_distance =
                distance;

            best = index;
        }
    }

    best as u8
}

fn chroma_strength(
    cb: f32,
    cr: f32,
) -> f32 {
    (cb * cb + cr * cr)
        .sqrt()
}

/* -------------------------------------------------------------------------- */
/* ANSI output                                                               */
/* -------------------------------------------------------------------------- */

fn push_color(
    output: &mut String,
    mode: u8,
    color: u8,
) {
    output.push_str("\x1b[");

    push_number(
        output,
        mode,
    );

    output.push_str(";5;");

    push_number(
        output,
        color,
    );

    output.push('m');
}

fn push_number(
    output: &mut String,
    value: u8,
) {
    if value >= 100 {
        output.push(
            (b'0' + value / 100)
                as char,
        );

        output.push(
            (b'0'
                + (value / 10) % 10)
                as char,
        );

        output.push(
            (b'0' + value % 10)
                as char,
        );
    } else if value >= 10 {
        output.push(
            (b'0' + value / 10)
                as char,
        );

        output.push(
            (b'0' + value % 10)
                as char,
        );
    } else {
        output.push(
            (b'0' + value)
                as char,
        );
    }
}

/* -------------------------------------------------------------------------- */
/* Helpers                                                                    */
/* -------------------------------------------------------------------------- */

fn to_byte(
    value: f32,
) -> u8 {
    (value.clamp(0.0, 1.0)
        * 255.0) as u8
}