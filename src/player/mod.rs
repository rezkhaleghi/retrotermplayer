use std::fs::OpenOptions;
use std::io::{self, Read};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::{decoder::FfmpegDecoder, renderer::Renderer, terminal::Terminal};

struct RawMode {
    original_settings: String,
}

impl RawMode {
    fn enter() -> Result<Self, String> {
        let output = Command::new("stty")
            .args(["-g"])
            .stdin(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("/dev/tty")
                    .map_err(|error| format!("Failed to open terminal: {error}"))?,
            )
            .output()
            .map_err(|error| format!("Failed to read terminal settings: {error}"))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to read terminal settings: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let original_settings = String::from_utf8(output.stdout)
            .map_err(|error| format!("Invalid terminal settings: {error}"))?
            .trim()
            .to_string();

        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(|error| format!("Failed to open terminal: {error}"))?;

        let status = Command::new("stty")
            .args(["-icanon", "-echo", "min", "0", "time", "0"])
            .stdin(tty)
            .status()
            .map_err(|error| format!("Failed to configure keyboard input: {error}"))?;

        if !status.success() {
            return Err("Failed to configure terminal keyboard input.".to_string());
        }

        Ok(Self { original_settings })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        if let Ok(tty) = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
        {
            let _ = Command::new("stty")
                .arg(&self.original_settings)
                .stdin(tty)
                .status();
        }
    }
}

struct Keyboard {
    tty: std::fs::File,
}

impl Keyboard {
    fn new() -> Result<Self, String> {
        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(|error| format!("Failed to open terminal keyboard: {error}"))?;

        Ok(Self { tty })
    }

    fn read_input(&mut self) -> Option<Input> {
        let mut buffer = [0u8; 8];

        let bytes_read = self.tty.read(&mut buffer).ok()?;

        if bytes_read == 0 {
            return None;
        }

        match buffer[0] {
            b'q' | b'Q' => Some(Input::Quit),

            b' ' => Some(Input::Pause),

            // Left arrow: ESC [ D
            0x1b if bytes_read >= 3 && buffer[1] == b'[' && buffer[2] == b'D' => {
                Some(Input::SeekBackward)
            }

            // Right arrow: ESC [ C
            0x1b if bytes_read >= 3 && buffer[1] == b'[' && buffer[2] == b'C' => {
                Some(Input::SeekForward)
            }

            // Plain ESC.
            0x1b => Some(Input::Quit),

            _ => None,
        }
    }
}

enum Input {
    Quit,
    Pause,
    SeekBackward,
    SeekForward,
}

pub struct Player {
    decoder: FfmpegDecoder,
    renderer: Box<dyn Renderer>,
    terminal: Terminal,
    output: String,

    position: f64,
    duration: Option<f64>,
    paused: bool,
}

impl Player {
    pub fn new(
        decoder: FfmpegDecoder,
        renderer: Box<dyn Renderer>,
        terminal: Terminal,
        duration: Option<f64>,
    ) -> Self {
        Self {
            decoder,
            renderer,
            terminal,
            output: String::new(),
            position: 0.0,
            duration,
            paused: false,
        }
    }

    pub fn play(&mut self) -> Result<(), String> {
        self.terminal
            .enter()
            .map_err(|error| format!("Failed to enter terminal mode: {error}"))?;

        let raw_mode = match RawMode::enter() {
            Ok(mode) => mode,
            Err(error) => {
                let _ = self.terminal.leave();
                return Err(error);
            }
        };

        let mut keyboard = match Keyboard::new() {
            Ok(keyboard) => keyboard,
            Err(error) => {
                drop(raw_mode);
                let _ = self.terminal.leave();
                return Err(error);
            }
        };

        let playback_result = self.play_loop(&mut keyboard);

        drop(raw_mode);

        let leave_result = self
            .terminal
            .leave()
            .map_err(|error| format!("Failed to leave terminal mode: {error}"));

        match (playback_result, leave_result) {
            (Err(error), _) => Err(error),
            (Ok(()), Err(error)) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }

    fn play_loop(&mut self, keyboard: &mut Keyboard) -> Result<(), String> {
        let frame_duration = Duration::from_secs_f64(1.0 / self.decoder.fps() as f64);

        loop {
            if let Some(input) = keyboard.read_input() {
                match input {
                    Input::Quit => break,

                    Input::Pause => {
                        self.paused = !self.paused;
                    }

                    Input::SeekBackward => {
                        self.seek(-15.0)?;
                    }

                    Input::SeekForward => {
                        self.seek(15.0)?;
                    }
                }
            }

            if self.paused {
                thread::sleep(Duration::from_millis(30));
                continue;
            }

            let frame_start = Instant::now();

            let frame = match self.decoder.next_frame()? {
                Some(frame) => frame,
                None => break,
            };

            self.renderer.render(&frame, &mut self.output);

            self.position += frame_duration.as_secs_f64();

            if let Some(duration) = self.duration {
                if self.position > duration {
                    self.position = duration;
                }
            }

            self.append_status_line();

            self.terminal
                .draw(&self.output)
                .map_err(|error| format!("Failed to draw frame: {error}"))?;

            let elapsed = frame_start.elapsed();

            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }

        Ok(())
    }

    fn append_status_line(&mut self) {
        self.output.push_str("\x1b[90m");

        self.output.push_str(&format!(
            "  {} / {}",
            format_time(self.position),
            format_time(self.duration.unwrap_or(0.0))
        ));

        if self.paused {
            self.output.push_str("    [ PAUSED ]");
        }

        self.output
            .push_str("    ← →  SEEK 15s    SPACE  ");

        if self.paused {
            self.output.push_str("RESUME");
        } else {
            self.output.push_str("PAUSE");
        }

        self.output.push_str("    Q  QUIT");

        self.output.push_str("\x1b[0m\n");
    }

    fn seek(&mut self, offset: f64) -> Result<(), String> {
        let mut target = self.position + offset;

        if target < 0.0 {
            target = 0.0;
        }

        if let Some(duration) = self.duration {
            if target > duration {
                target = duration;
            }
        }

        self.decoder.seek_to(target)?;
        self.position = target;

        Ok(())
    }
}

fn format_time(seconds: f64) -> String {
    let seconds = seconds.max(0.0).round() as u64;

    let minutes = seconds / 60;
    let seconds = seconds % 60;

    format!("{minutes:02}:{seconds:02}")
}