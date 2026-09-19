use crate::decoder::VideoFrame;

use super::Renderer;

const CONTRAST: f32 = 1.08;
const BRIGHTNESS: f32 = 1.015;
const GAMMA: f32 = 0.94;

const SHARPEN: f32 = 0.30;
const LOCAL_CONTRAST: f32 = 0.10;
const TEMPORAL_STABILITY: f32 = 0.14;

/// High-resolution monochrome video renderer.
///
/// The decoder provides a 200x120 frame and the renderer preserves
/// the full horizontal resolution while using half-block characters.
///
/// Each `▀` represents two independent luminance pixels:
///
/// foreground = top pixel
/// background = bottom pixel
pub struct MonoVideoRenderer {
    previous: Vec<f32>,
    enhanced: Vec<f32>,
}

impl MonoVideoRenderer {
    pub fn new() -> Self {
        Self {
            previous: vec![0.0; 200 * 120],
            enhanced: vec![0.0; 200 * 120],
        }
    }

    fn convert_to_luminance(frame: &VideoFrame, output: &mut [f32]) {
        for y in 0..frame.height {
            for x in 0..frame.width {
                let index = y * frame.width + x;

                let (r, g, b) = frame.pixel(x, y);

                let r = r as f32 / 255.0;
                let g = g as f32 / 255.0;
                let b = b as f32 / 255.0;

                // Rec.709 luminance.
                output[index] = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            }
        }
    }

    fn process_image(&mut self, source: &[f32], width: usize, height: usize) {
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;

                let center = source[index];

                // 3x3 local average.
                let mut sum = 0.0;

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = clamp_coord(x as i32 + dx, width);

                        let ny = clamp_coord(y as i32 + dy, height);

                        sum += source[ny * width + nx];
                    }
                }

                let local_average = sum / 9.0;

                // High-frequency detail.
                let detail = center - local_average;

                // Subtle local contrast.
                let local = detail * LOCAL_CONTRAST;

                // Edge/detail enhancement.
                let mut value = center + detail * SHARPEN + local;

                // Stabilize only extremely small changes.
                let previous = self.previous[index];

                if (value - previous).abs() < 0.018 {
                    value = value * (1.0 - TEMPORAL_STABILITY) + previous * TEMPORAL_STABILITY;
                }

                // Global contrast.
                value = (value - 0.5) * CONTRAST + 0.5;

                // Brightness.
                value *= BRIGHTNESS;

                // Keep full dynamic range.
                value = value.clamp(0.0, 1.0);

                // Slight gamma adjustment.
                value = value.powf(GAMMA);

                self.enhanced[index] = value.clamp(0.0, 1.0);
            }
        }

        self.previous[..width * height].copy_from_slice(&self.enhanced[..width * height]);
    }
}

impl Default for MonoVideoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for MonoVideoRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        let pixel_count = frame.width * frame.height;

        let mut luminance = vec![0.0; pixel_count];

        Self::convert_to_luminance(frame, &mut luminance);

        self.process_image(&luminance, frame.width, frame.height);

        output.clear();
        output.push_str("\x1b[H");

        // Each half-block character represents two vertical pixels.
        // We preserve the full horizontal resolution.
        for y in (0..frame.height).step_by(2) {
            let bottom_y = (y + 1).min(frame.height - 1);

            let mut current_fg: Option<u8> = None;
            let mut current_bg: Option<u8> = None;

            for x in 0..frame.width {
                let top = self.enhanced[y * frame.width + x];

                let bottom = self.enhanced[bottom_y * frame.width + x];

                let fg = grayscale_ansi(top);

                let bg = grayscale_ansi(bottom);

                if current_fg != Some(fg) {
                    push_color(output, 38, fg);

                    current_fg = Some(fg);
                }

                if current_bg != Some(bg) {
                    push_color(output, 48, bg);

                    current_bg = Some(bg);
                }

                output.push('▀');
            }

            output.push_str("\x1b[0m\n");
        }
    }
}

fn grayscale_ansi(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);

    let index = 232.0 + value * 23.0;

    index.round().clamp(232.0, 255.0) as u8
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

fn clamp_coord(value: i32, limit: usize) -> usize {
    value.clamp(0, limit as i32 - 1) as usize
}
