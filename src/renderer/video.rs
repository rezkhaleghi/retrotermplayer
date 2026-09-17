use crate::decoder::VideoFrame;

use super::Renderer;

/// Normal-quality terminal video renderer.
///
/// Uses ANSI truecolor and half-block characters. Each terminal cell
/// represents two vertical pixels, giving substantially better detail
/// than one-character-per-pixel rendering while keeping the output
/// manageable for a normal terminal.
pub struct VideoRenderer;

impl VideoRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for VideoRenderer {
    fn render(&mut self, frame: &VideoFrame) -> String {
        let mut output = String::with_capacity(frame.width * frame.height * 12);

        output.push_str("\x1b[H");

        for y in (0..frame.height).step_by(2) {
            let mut current_top = None;
            let mut current_bottom = None;

            for x in 0..frame.width {
                let top = frame.pixel(x, y);

                let bottom = if y + 1 < frame.height {
                    frame.pixel(x, y + 1)
                } else {
                    (0, 0, 0)
                };

                if current_top != Some(top) {
                    output.push_str(&format!(
                        "\x1b[38;2;{};{};{}m",
                        top.0, top.1, top.2
                    ));

                    current_top = Some(top);
                }

                if current_bottom != Some(bottom) {
                    output.push_str(&format!(
                        "\x1b[48;2;{};{};{}m",
                        bottom.0, bottom.1, bottom.2
                    ));

                    current_bottom = Some(bottom);
                }

                output.push('▀');
            }

            output.push_str("\x1b[0m\n");
        }

        output
    }
}