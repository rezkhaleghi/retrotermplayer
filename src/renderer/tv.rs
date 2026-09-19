use crate::decoder::VideoFrame;

use super::Renderer;

const PANEL_WIDTH: usize = 23;
const PANEL_CONTENT_WIDTH: usize = 18;

/// Wraps any visual renderer inside the RetroTermPlayer CRT television.
pub struct TvRenderer {
    inner: Box<dyn Renderer>,
    screen: String,
}

impl TvRenderer {
    pub fn new(inner: Box<dyn Renderer>) -> Self {
        Self {
            inner,
            screen: String::new(),
        }
    }
}

impl Renderer for TvRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        self.screen.clear();
        self.inner.render(frame, &mut self.screen);

        output.clear();

        let screen = self.screen.strip_prefix("\x1b[H").unwrap_or(&self.screen);

        let screen_width = frame.width;
        let screen_height = frame.height.div_ceil(2);

        // Screen section:
        //
        // ║  VIDEO  ║
        //
        // 1 + 2 + video + 2 + 1
        let screen_row_width = screen_width + 6;

        // Complete TV width:
        //
        // screen section + control panel
        let total_width = screen_row_width + PANEL_WIDTH;

        render_top(output, total_width);
        output.push('\n');

        render_brand_row(output, screen_row_width);
        render_panel_header(output);
        output.push('\n');

        render_bezel_top(output, screen_width);
        render_panel_separator(output);
        output.push('\n');

        let mut lines = screen.lines();

        for row in 0..screen_height {
            // Left screen frame.
            output.push('║');
            output.push_str("  ");

            if let Some(line) = lines.next() {
                if row % 2 == 1 {
                    render_scanline(output, line);
                } else {
                    output.push_str(line);
                }

                let visible = visible_width(line);

                if visible < screen_width {
                    output.push_str(&" ".repeat(screen_width - visible));
                }
            } else {
                output.push_str(&" ".repeat(screen_width));
            }

            output.push_str("  ");
            output.push('║');

            // Right control panel.
            render_controls(output, row);

            output.push('\n');
        }

        render_bezel_bottom(output, screen_width);
        render_panel_bottom(output);
        output.push('\n');

        render_bottom_panel(output, total_width);
        output.push('\n');

        render_bottom(output, total_width);
        output.push('\n');
    }
}

fn render_top(output: &mut String, total_width: usize) {
    output.push_str("\x1b[90m╔");
    output.push_str(&"═".repeat(total_width - 2));
    output.push_str("╗\x1b[0m");
}

fn render_brand_row(output: &mut String, screen_row_width: usize) {
    output.push_str("\x1b[90m║");
    output.push_str("  ");

    let left = "RETROTERM";
    let right = "CRT-480";

    // Content between the outer left/right borders.
    let content_width = screen_row_width - 2;

    output.push_str("\x1b[97m");
    output.push_str(left);

    let remaining = content_width.saturating_sub(
        2 + left.len() + right.len() + 2,
    );

    output.push_str(&" ".repeat(remaining));
    output.push_str(right);

    output.push_str("\x1b[90m  ║\x1b[0m");
}




fn render_panel_header(output: &mut String) {
    // Exactly PANEL_WIDTH cells:
    //
    // 2 spaces + ┌ + 18 dashes + ┐ + ║
    output.push_str("\x1b[90m  ┌──────────────────┐║\x1b[0m");
}

fn render_bezel_top(output: &mut String, screen_width: usize) {
    output.push_str("\x1b[90m║");

    output.push_str(" ╭");
    output.push_str(&"─".repeat(screen_width));
    output.push_str("╮ ║");
}

fn render_panel_separator(output: &mut String) {
    // Exactly PANEL_WIDTH cells.
    output.push_str("\x1b[90m  │                  │║\x1b[0m");
}

fn render_bezel_bottom(output: &mut String, screen_width: usize) {
    output.push_str("\x1b[90m║");

    output.push_str(" ╰");
    output.push_str(&"─".repeat(screen_width));
    output.push_str("╯ ║");
}

fn render_panel_bottom(output: &mut String) {
    // Exactly PANEL_WIDTH cells.
    output.push_str("\x1b[90m  └──────────────────┘║\x1b[0m");
}

fn render_scanline(output: &mut String, line: &str) {
    output.push_str("\x1b[2m");
    output.push_str(line);
    output.push_str("\x1b[22m");
}

fn render_controls(output: &mut String, row: usize) {
    output.push_str("\x1b[90m");

    let content = match row {
        0 => "● REC",
        1 => "▶ PLAY",
        2 => "",
        3 => "     ◉    ◉",
        4 => "    VOL   CH",
        5 => "",
        6 => "  ▒▒▒▒▒▒▒▒▒▒▒",
        7 => "  ▒▒▒▒▒▒▒▒▒▒▒",
        8 => "",
        _ => "",
    };

    // The panel has exactly:
    //
    // 2 spaces + │ + 18 content cells + │ + ║
    //
    // = 23 cells.
    output.push_str("  │ ");

    let content_width = PANEL_CONTENT_WIDTH - 1;
    let visible = visible_width(content);

    output.push_str(content);

    if visible < content_width {
        output.push_str(&" ".repeat(content_width - visible));
    }

    output.push_str("│║");
    output.push_str("\x1b[0m");
}

fn render_bottom_panel(output: &mut String, total_width: usize) {
    output.push_str("\x1b[90m║");
    output.push_str("  ");

    // Full row:
    //
    // ║ + 2 spaces + speaker + 2 spaces + ║
    //
    // Total = total_width.
    let speaker_width = total_width.saturating_sub(6);

    for index in 0..speaker_width {
        let character = match index % 6 {
            0 | 1 => '·',
            2 | 3 => '•',
            _ => ' ',
        };

        output.push(character);
    }

    output.push_str("  ║\x1b[0m");
}

fn render_bottom(output: &mut String, total_width: usize) {
    output.push_str("\x1b[90m╚");
    output.push_str(&"═".repeat(total_width - 2));
    output.push_str("╝\x1b[0m");
}

fn visible_width(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut index = 0;
    let mut width = 0;

    while index < bytes.len() {
        if bytes[index] == 0x1b {
            index += 1;

            if index < bytes.len() && bytes[index] == b'[' {
                index += 1;

                while index < bytes.len() {
                    let byte = bytes[index];
                    index += 1;

                    if (0x40..=0x7e).contains(&byte) {
                        break;
                    }
                }

                continue;
            }

            continue;
        }

        let character = line[index..]
            .chars()
            .next()
            .expect("valid UTF-8 character");

        width += 1;
        index += character.len_utf8();
    }

    width
}
