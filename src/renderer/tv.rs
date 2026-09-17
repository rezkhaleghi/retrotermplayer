use crate::decoder::VideoFrame;

use super::Renderer;

const PANEL_WIDTH: usize = 10;

/// Reusable CRT television wrapper.
///
/// The wrapped renderer provides only the picture. This renderer owns the
/// complete television cabinet around it.
pub struct TvRenderer {
    inner: Box<dyn Renderer>,
}

impl TvRenderer {
    pub fn new(inner: Box<dyn Renderer>) -> Self {
        Self { inner }
    }
}

impl Renderer for TvRenderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String) {
        let mut screen = String::new();

        self.inner.render(frame, &mut screen);

        output.clear();

        let screen = screen.strip_prefix("\x1b[H").unwrap_or(&screen);

        let screen_width = frame.width;
        let screen_height = (frame.height + 1) / 2;

        // The actual screen rows have:
        //
        // ║ + 2 spaces + screen + 2 spaces + ║
        //
        // Keep every cabinet row based on exactly the same width.
        let screen_row_width = screen_width + 4;

        // Top frame.
        output.push('╔');
        output.push_str(&"═".repeat(screen_row_width + PANEL_WIDTH + 1));
        output.push('╗');
        output.push('\n');

        // Blank row above the screen.
        render_blank_row(output, screen_row_width);
        output.push('\n');

        let mut lines = screen.lines();

        // Screen.
        for row in 0..screen_height {
            output.push('║');

            // Left screen margin.
            output.push_str("  ");

            if let Some(line) = lines.next() {
                output.push_str(line);

                let visible = visible_width(line);

                if visible < screen_width {
                    output.push_str(&" ".repeat(screen_width - visible));
                }
            } else {
                output.push_str(&" ".repeat(screen_width));
            }

            // Right screen margin.
            output.push_str("  ");

            output.push('║');

            render_wood_panel(output, row);

            output.push('\n');
        }

        // Blank row below the screen.
        render_blank_row(output, screen_row_width);
        output.push('\n');

        // Bottom textured cabinet.
        output.push('║');

        let total_width = screen_row_width + PANEL_WIDTH + 1;
        render_texture(output, total_width);

        output.push('║');
        output.push('\n');

        // Bottom frame.
        output.push('╚');
        output.push_str(&"═".repeat(total_width));
        output.push('╝');
        output.push('\n');
    }
}

/// Blank cabinet row.
///
/// This intentionally uses exactly the same width as the screen area:
///
/// ```text
/// ║  <screen>  ║
/// ```
///
/// but replaces the screen with spaces.
fn render_blank_row(output: &mut String, screen_row_width: usize) {
    output.push('║');
    output.push_str(&" ".repeat(screen_row_width));
    output.push('║');

    output.push_str(&" ".repeat(PANEL_WIDTH));

    output.push('║');
}

/// Wooden side panel.
///
/// The panel is always exactly PANEL_WIDTH characters wide.
fn render_wood_panel(output: &mut String, row: usize) {
    let pattern = match row % 8 {
        0 => "──────────",
        1 => "── ~~~~~ ─",
        2 => "──────────",
        3 => "─ ~────~ ─",
        4 => "──────────",
        5 => "~~~ ──────",
        6 => "──────────",
        _ => "─ ─── ~~~~",
    };

    let mut chars = pattern.chars();

    for index in 0..PANEL_WIDTH {
        // Replace an existing character with the dot.
        if row % 11 == 0 && index == 8 {
            output.push('●');
        } else {
            output.push(chars.next().unwrap_or(' '));
        }
    }

    output.push('║');
}

/// Repeating old-TV texture used at the bottom of the cabinet.
///
/// The function writes exactly `width` terminal characters.
fn render_texture(output: &mut String, width: usize) {
    let pattern = ['░', '▒', '▓'];
    let mut index = 0;

    for _ in 0..width {
        output.push(pattern[index]);
        index = (index + 1) % pattern.len();
    }
}

/// Calculates the visible terminal width of a line.
///
/// ANSI escape sequences are ignored. This is required for the Color and
/// VHS renderers because their output contains ANSI color control sequences.
fn visible_width(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut index = 0;
    let mut width = 0;

    while index < bytes.len() {
        if bytes[index] == 0x1b {
            index += 1;

            // CSI sequence: ESC [
            if index < bytes.len() && bytes[index] == b'[' {
                index += 1;

                while index < bytes.len() {
                    let byte = bytes[index];
                    index += 1;

                    // CSI final byte.
                    if (0x40..=0x7e).contains(&byte) {
                        break;
                    }
                }

                continue;
            }

            continue;
        }

        let character = line[index..].chars().next().expect("valid UTF-8 character");

        width += 1;
        index += character.len_utf8();
    }

    width
}
