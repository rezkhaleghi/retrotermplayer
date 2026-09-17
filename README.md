# RetroTermPlayer

A small, dependency-free Rust terminal video renderer/player.

RetroTermPlayer takes a local video file, direct media URL, or YouTube URL, decodes it through **FFmpeg**, and renders the video directly inside a terminal using ANSI escape sequences.

The project is intentionally simple and modular so the rendering engine can eventually be reused inside other projects such as **PJ-PLAYER**.

---

## Features

- 🎞️ Play local video files
- 🌐 Play direct media URLs
- ▶️ Play YouTube videos through `yt-dlp`
- 🖥️ Render video directly inside the terminal
- 🎨 Truecolor terminal rendering
- 🔤 Retro ASCII rendering
- 📺 Retro ANSI color rendering
- 📼 VHS-style rendering
- 🎬 Normal terminal video rendering
- ⚡ No Rust runtime dependencies
- 🔧 Uses external `ffmpeg` and `yt-dlp`
- 🧩 Modular source / decoder / renderer / player architecture
- 🧪 Compare all four renderers simultaneously with `compare.sh`

---

# How It Works

RetroTermPlayer uses a simple media pipeline:

```text
                ┌──────────────┐
                │ Video Source │
                └──────┬───────┘
                       │
             ┌─────────┴─────────┐
             │                   │
          Local               YouTube
             │                   │
             │                yt-dlp
             │                   │
             └─────────┬─────────┘
                       │
                       ▼
                ┌─────────────┐
                │   FFmpeg    │
                │   Decoder   │
                └──────┬──────┘
                       │
                   RGB frames
                       │
                       ▼
                ┌─────────────┐
                │   Player    │
                └──────┬──────┘
                       │
                       ▼
                 ┌───────────┐
                 │ Renderer  │
                 └─────┬─────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
      ASCII          Color           VHS
                                       │
                                       ▼
                                     Video
```

The important separation is:

```text
Source → Decoder → Player → Renderer → Terminal
```

Each part has a specific responsibility.

---

# Requirements

## Rust

Install Rust through `rustup` if it is not already installed.

Check:

```bash
rustc --version
cargo --version
```

---

## FFmpeg

FFmpeg performs the actual video decoding and frame conversion.

Check:

```bash
ffmpeg -version
```

### macOS

Using Homebrew:

```bash
brew install ffmpeg
```

---

## yt-dlp

`yt-dlp` is required for YouTube URLs.

Check:

```bash
yt-dlp --version
```

### macOS

```bash
brew install yt-dlp
```

You can also install it through Python/pip or another supported method.

---

# Installation

Clone the repository:

```bash
git clone https://github.com/rezkhaleghi/retrotermplayer.git
cd retrotermplayer
```

Build:

```bash
cargo build
```

Run the project:

```bash
cargo run -- <source> <renderer>
```

---

# Usage

The basic syntax is:

```bash
cargo run -- <video-url-or-file-path> <renderer>
```

The renderer is a number from `1` to `4`.

| Renderer | Mode        |
| -------- | ----------- |
| `1`      | Retro ASCII |
| `2`      | Retro Color |
| `3`      | VHS         |
| `4`      | Video       |

---

# Renderer 1 — Retro ASCII

```bash
cargo run -- "VIDEO_SOURCE" 1
```

The video is converted into a monochrome-style terminal representation using block characters.

Example:

```text
████████████████████████████████████
██████████▀▀▀▀▀▀▀▀▀▀████████████████
██████▀▀              ▀▀████████████
████                      ██████████
███                        █████████
████                      ██████████
██████▀▀              ▀▀████████████
██████████▄▄▄▄▄▄▄▄▄▄████████████████
████████████████████████████████████
```

This mode intentionally sacrifices color and detail for a strong terminal / ASCII aesthetic.

### Characteristics

- Low rendering overhead
- Low terminal bandwidth
- High contrast
- Monochrome/block-based appearance
- Good for small terminals
- Retro computer aesthetic

---

# Renderer 2 — Retro Color

```bash
cargo run -- "VIDEO_SOURCE" 2
```

Uses ANSI 256-color rendering.

The renderer uses terminal half-block characters:

```text
▀
```

One terminal cell represents two vertical pixels:

```text
┌───────────────┐
│ upper pixel   │
│      ▀        │
│ lower pixel   │
└───────────────┘
```

The foreground color represents the upper pixel and the background color represents the lower pixel.

This effectively doubles the vertical visual resolution compared with using one character per pixel.

### Characteristics

- ANSI 256 colors
- Half-block rendering
- Better color reproduction than ASCII
- Still relatively lightweight
- Strong retro terminal appearance

---

# Renderer 3 — VHS

```bash
cargo run -- "VIDEO_SOURCE" 3
```

The VHS renderer starts with color terminal rendering and adds visual degradation intended to resemble old analog video.

The goal is not to destroy the image.

Instead, it aims for subtle imperfections such as:

- Scanlines
- Slight brightness fluctuations
- Small color inconsistencies
- Mild tracking distortion
- Analog-style visual noise
- Occasional frame instability

The exact effects are intentionally implemented in the renderer rather than modifying the source video itself.

This keeps the original decoded frame available to other renderers.

---

# Renderer 4 — Video

```bash
cargo run -- "VIDEO_SOURCE" 4
```

The Video renderer is the closest mode to normal video playback.

It uses:

- RGB frames
- ANSI truecolor
- Half-block rendering
- Higher rendering resolution
- Higher frame rate

Example ANSI colors:

```text
\x1b[38;2;255;0;0m
\x1b[48;2;0;0;255m
```

The terminal therefore receives actual RGB color values instead of a reduced 256-color palette.

### Important

This is still terminal video.

It is not equivalent to playing the video in VLC, QuickTime, mpv, or a graphical video player.

The terminal imposes several limitations:

- Terminal dimensions
- Character-cell geometry
- ANSI output bandwidth
- Terminal rendering performance
- CPU usage
- Source resolution

The renderer therefore targets the available terminal resolution rather than attempting to render an arbitrary 1080p/4K frame directly.

---

# Video Sources

RetroTermPlayer supports three source types.

## 1. Local files

You can provide a normal filesystem path:

```bash
cargo run -- "/Users/reza/Movies/video.mp4" 4
```

Home-directory paths are also supported:

```bash
cargo run -- "~/Movies/video.mp4" 4
```

The source resolver automatically detects existing files.

---

## 2. Direct media URLs

HTTP/HTTPS URLs can be passed directly:

```bash
cargo run -- "https://example.com/video.mp4" 4
```

RTMP and RTSP sources are also recognized:

```bash
cargo run -- "rtmp://example.com/live" 4
```

```bash
cargo run -- "rtsp://example.com/live" 4
```

FFmpeg handles the actual media decoding.

---

## 3. YouTube

YouTube URLs are resolved through `yt-dlp`.

Example:

```bash
cargo run -- "https://www.youtube.com/watch?v=WvV5TbJc9tQ" 4
```

The architecture deliberately keeps YouTube-specific logic inside the source layer.

The rest of the application does not need to know that the source came from YouTube.

---

# YouTube Source Resolution

The YouTube pipeline is:

```text
YouTube URL
     │
     ▼
   yt-dlp
     │
     ▼
Direct media stream URL
     │
     ▼
   FFmpeg
     │
     ▼
RGB frames
```

This means the renderer does not depend on YouTube.

It receives exactly the same `VideoFrame` regardless of whether the original source was:

```text
local.mp4
```

or:

```text
https://example.com/video.mp4
```

or:

```text
https://youtube.com/watch?v=...
```

That separation is important for keeping the rendering engine reusable.

---

# Comparing All Four Modes

The project includes a helper script for comparing the four renderers simultaneously.

The root directory contains:

```text
compare.sh
```

The script currently uses:

```text
https://www.youtube.com/watch?v=WvV5TbJc9tQ&list=RDWvV5TbJc9tQ&start_radio=1
```

Make sure it is executable:

```bash
chmod +x compare.sh
```

Then simply run:

```bash
./compare.sh
```

On macOS this opens four Terminal windows.

Each window runs one renderer:

```text
Terminal 1 → Retro ASCII
Terminal 2 → Retro Color
Terminal 3 → VHS
Terminal 4 → Video
```

This is useful when tuning renderer quality because all four modes can be observed using the same source.

### Performance note

Running four instances simultaneously means:

```text
4 × FFmpeg
4 × Rust renderer
4 × terminal output
```

Therefore this comparison is intended primarily for **visual comparison**, not performance benchmarking.

CPU usage will naturally be much higher.

---

# Architecture

The project is intentionally split into independent layers.

```text
src/
├── main.rs
├── lib.rs
│
├── input/
│   └── mod.rs
│
├── source/
│   ├── mod.rs
│   ├── direct.rs
│   ├── local.rs
│   └── youtube.rs
│
├── decoder/
│   ├── mod.rs
│   └── ffmpeg.rs
│
├── renderer/
│   ├── mod.rs
│   ├── ascii.rs
│   ├── color.rs
│   ├── vhs.rs
│   └── video.rs
│
├── player/
│   └── mod.rs
│
└── terminal/
    └── mod.rs
```

---

# Input

```text
src/input/
```

Responsible for command-line input.

Example:

```bash
cargo run -- "video.mp4" 2
```

It produces:

```rust
Input {
    source,
    renderer,
}
```

Interactive startup input is intentionally avoided.

This means the application does not need to consume stdin just to select the video and renderer.

That also keeps stdin available for future playback controls.

---

# Source

```text
src/source/
```

Responsible for identifying and resolving media sources.

Current source types:

```rust
VideoSource::Local
VideoSource::Direct
VideoSource::YouTube
```

The source layer hides source-specific details from the decoder.

For example, the decoder does not need to know how YouTube URLs are resolved.

---

# Decoder

```text
src/decoder/
```

Responsible for starting FFmpeg and converting the media stream into raw RGB frames.

The decoder outputs:

```rust
VideoFrame {
    width,
    height,
    pixels,
}
```

The frame contains:

```text
RGB RGB RGB RGB ...
```

with three bytes per pixel.

Conceptually:

```text
Video
  ↓
FFmpeg
  ↓
RGB24
  ↓
VideoFrame
```

---

# Decoder Profiles

Different renderers have different requirements.

A retro renderer does not need the same source resolution or frame rate as the normal video renderer.

The decoder therefore supports profiles.

Conceptually:

```text
RETRO
├── small resolution
└── lower FPS

VHS
├── small/medium resolution
└── lower FPS

VIDEO
├── larger resolution
└── higher FPS
```

This prevents every renderer from unnecessarily processing large frames.

---

# Player

```text
src/player/
```

The player controls playback timing.

Its responsibility is:

```text
decode frame
     ↓
render frame
     ↓
draw frame
     ↓
wait for target frame interval
     ↓
next frame
```

The player does not know how a frame is rendered.

It only knows that a renderer implements:

```rust
trait Renderer {
    fn render(&mut self, frame: &VideoFrame) -> String;
}
```

This makes it possible to add new rendering modes without changing the playback engine.

---

# Renderer

```text
src/renderer/
```

Contains the visual presentation layer.

Current renderers:

```text
AsciiRenderer
ColorRenderer
VhsRenderer
VideoRenderer
```

All implement the same interface:

```rust
pub trait Renderer {
    fn render(&mut self, frame: &VideoFrame) -> String;
}
```

This means the player can work with any renderer.

Adding a new renderer should not require modifying:

- FFmpeg
- source resolution
- player timing
- terminal management

Only the renderer needs to be added.

---

# Terminal

```text
src/terminal/
```

Responsible for terminal-specific operations:

- Clearing the terminal
- Moving the cursor
- Hiding the cursor
- Writing rendered frames
- Restoring the cursor
- Resetting terminal colors

The renderer produces ANSI output.

The terminal layer is responsible for displaying it.

---

# Why Half-Block Rendering?

A normal terminal character occupies one cell:

```text
┌───┐
│ A │
└───┘
```

A half-block character allows two vertical pixels to be represented in one cell:

```text
▀
```

The upper half uses the foreground color.

The lower half uses the background color.

Therefore:

```text
RGB pixel
RGB pixel
```

becomes:

```text
▀
```

with two different colors.

This gives the player considerably more vertical visual information than ordinary ASCII rendering.

---

# Color Modes

## ANSI 256 Color

Retro Color and VHS use the terminal's 256-color palette.

The renderer converts:

```text
RGB 0-255
```

into:

```text
ANSI color 0-255
```

This reduces color precision but significantly reduces the amount of color information that must be represented.

---

## ANSI Truecolor

The Video renderer uses:

```text
RGB 24-bit
```

through ANSI truecolor sequences.

Example:

```text
ESC[38;2;255;100;50m
```

This allows the renderer to preserve substantially more color information.

Truecolor support depends on the terminal emulator.

Modern macOS terminals generally support it.

---

# Terminal Resolution

The terminal itself is one of the biggest limitations.

For example, a terminal with:

```text
100 columns × 25 rows
```

cannot display a 1920×1080 image at its original resolution.

RetroTermPlayer therefore scales the source video before sending frames to the terminal.

The general pipeline is:

```text
1920 × 1080 source
        ↓
FFmpeg scaling
        ↓
terminal-sized RGB frame
        ↓
half-block renderer
        ↓
100 × 25 terminal
```

The source resolution and terminal resolution are therefore separate concepts.

A higher-quality source does not automatically produce more terminal pixels.

---

# Why Normal Video Can Still Look Low Quality

There are several independent limits.

## 1. Terminal resolution

A terminal might only provide:

```text
100 × 25
```

cells.

---

## 2. Frame size

The decoder intentionally scales the video to a terminal-friendly size.

---

## 3. ANSI output

Every rendered frame becomes a large string of ANSI escape sequences.

For example:

```text
RGB frame
    ↓
color escape codes
    ↓
terminal characters
    ↓
stdout
    ↓
terminal emulator
```

This can become expensive at high resolutions and frame rates.

---

## 4. Terminal rendering speed

The terminal itself must parse and draw every ANSI sequence.

At sufficiently high frame rates, the terminal can become the bottleneck.

---

## 5. Source quality

The quality of the original stream also matters.

A low-quality YouTube stream cannot be reconstructed into a high-quality image simply by increasing the terminal resolution.

---

# Why FFmpeg Is External

FFmpeg is intentionally not implemented inside Rust.

FFmpeg already provides:

- Container parsing
- Codec support
- Hardware acceleration
- Scaling
- Frame-rate conversion
- Network protocols
- Image conversion
- Hundreds of video/audio codecs

Reimplementing that functionality would make the project significantly larger and less reliable.

RetroTermPlayer therefore uses FFmpeg as a media backend.

Rust handles:

```text
application architecture
source management
playback
rendering
terminal interaction
```

FFmpeg handles:

```text
media decoding
```

---

# Why There Are No Rust Dependencies

The current `Cargo.toml` intentionally contains no runtime dependencies.

```toml
[dependencies]
```

The goal is to keep the core player lightweight.

External tools are allowed:

```text
ffmpeg
yt-dlp
```

The distinction is intentional:

```text
Rust dependency
    ↓
compiled into application

External media tool
    ↓
replaceable backend
```

This also makes it easier to experiment with different media backends in the future.

---

# Current Limitations

RetroTermPlayer is still an experimental player.

Current limitations include:

- Terminal resolution limits visual quality
- Terminal output can become CPU-intensive
- High resolutions produce large ANSI output
- Four simultaneous players consume significant CPU
- Audio playback is not currently implemented
- YouTube currently relies on `yt-dlp`
- FFmpeg must be installed separately
- yt-dlp must be installed for YouTube playback
- Playback controls are not yet fully implemented
- Terminal truecolor support varies between terminals
- Network sources depend on network stability
- Some URLs may not be supported directly by FFmpeg
- Some websites may require yt-dlp rather than direct FFmpeg access

---

# Audio

Audio is intentionally not part of the current rendering pipeline.

The current pipeline is:

```text
Source
  ↓
FFmpeg
  ↓
RGB video frames
  ↓
Renderer
  ↓
Terminal
```

There is currently no:

```text
audio decoder
audio output
audio synchronization
```

This is deliberate.

The project started as a visual terminal renderer.

Audio can be introduced later without requiring the renderers themselves to understand audio.

A future architecture could look like:

```text
                    ┌──→ Video Decoder ──→ Renderer ──→ Terminal
Source ──→ Player ──┤
                    └──→ Audio Decoder ──→ Audio Output
```

---

# Future Direction

RetroTermPlayer is intended to evolve into a reusable terminal media engine.

Potential future features include:

- Playback controls
- Pause / resume
- Seeking
- Frame stepping
- Volume control
- Audio playback
- Audio/video synchronization
- Terminal-size detection
- Better frame scheduling
- More sophisticated VHS effects
- More rendering modes
- Braille rendering
- Unicode shading
- Dithering
- Truecolor image optimization
- Hardware-accelerated decoding where available
- Better YouTube format selection
- Broader yt-dlp source support
- Reusable integration into PJ-PLAYER

---

# Relationship With PJ-PLAYER

RetroTermPlayer is intentionally designed so its rendering system can eventually be reused by [PJ-PLAYER](https://github.com/rezkhaleghi/pj-player).

PJ-PLAYER is primarily a terminal music player.

RetroTermPlayer explores the video side of terminal media playback.

The long-term concept is:

```text
                 PJ-PLAYER
                     │
              ┌──────┴──────┐
              │             │
           Audio          Video
              │             │
              │      RetroTermPlayer
              │             │
              ▼             ▼
           Audio          Terminal
           Output          Renderer
```

The projects can remain separate while their reusable components mature.

---

# Development

Build:

```bash
cargo build
```

Run:

```bash
cargo run -- "video.mp4" 1
```

Run with a YouTube URL:

```bash
cargo run -- "https://www.youtube.com/watch?v=WvV5TbJc9tQ" 4
```

Check formatting:

```bash
cargo fmt --check
```

Format:

```bash
cargo fmt
```

Run tests:

```bash
cargo test
```

Check the project:

```bash
cargo check
```

---

# Project Philosophy

RetroTermPlayer is deliberately small.

The goal is not to build another FFmpeg.

The goal is to build a clean Rust layer around existing media tools and explore how far terminal rendering can be pushed.

The architecture follows a simple rule:

> **Each component should do one thing and know as little as possible about the others.**

Source handling should not know about rendering.

Rendering should not know about YouTube.

The player should not know about ANSI color implementation.

The terminal should not know where the video came from.

That separation is what makes the project useful beyond the current prototype.

---

# License

License information will be added when the project is ready for its first public release.
