# RetroTermPlayer

A small Rust terminal video player that renders video directly inside the terminal.

RetroTermPlayer uses **FFmpeg** for decoding and supports local files, direct media URLs, and YouTube videos through **yt-dlp**.

The project is intentionally lightweight and modular, with the rendering engine designed to eventually be reusable inside projects such as **PJ-PLAYER**.

---

## Features

- 🎞️ Local video playback
- 🌐 Direct HTTP/HTTPS media URLs
- ▶️ YouTube playback through `yt-dlp`
- 🖥️ Video rendered directly in the terminal
- 📺 CRT / retro TV cabinet around the video
- 🔤 Retro ASCII renderer
- 🎨 Retro color renderer
- 📼 VHS-style renderer
- 🎬 Truecolor video renderer
- 📁 Interactive offline browser
- 🔎 Recursive video search
- `~` and `$HOME` path expansion
- Quoted paths supported
- No Rust runtime dependencies
- FFmpeg-based decoding
- Modular source / decoder / player / renderer architecture

---

# How It Works

The core pipeline is intentionally simple:

```text
Video Source
     │
     ▼
Source Resolver
     │
     ▼
FFmpeg Decoder
     │
     ▼
RGB Video Frames
     │
     ▼
Player
     │
     ▼
Renderer
     │
     ▼
CRT / TV Wrapper
     │
     ▼
Terminal
```

Source-specific logic stays inside the source layer.

The renderer does not need to know whether the video came from a local file, a URL, or YouTube.

---

# Requirements

## Rust

Install Rust through [rustup](https://rustup.rs/) if it is not already installed.

Check:

```bash
rustc --version
cargo --version
```

## FFmpeg

FFmpeg performs the actual video decoding and frame conversion.

Check:

```bash
ffmpeg -version
```

### macOS

```bash
brew install ffmpeg
```

## yt-dlp

`yt-dlp` is required for YouTube playback.

Check:

```bash
yt-dlp --version
```

### macOS

```bash
brew install yt-dlp
```

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

---

# Usage

RetroTermPlayer supports both an interactive interface and the original command-line interface.

## Interactive Mode

Run:

```bash
cargo run
```

You will see:

```text
╔══════════════════════════════════════╗
║          RETROTERMPLAYER             ║
╚══════════════════════════════════════╝

1. Online
2. Offline

Select:
```

### Online

Select `1` and enter a video URL:

```text
ONLINE

Video URL:
```

HTTP and HTTPS URLs are supported.

YouTube URLs are automatically detected and resolved through `yt-dlp`.

### Offline

Select `2`:

```text
OFFLINE

1. Enter path
2. Browse
3. Search

Select:
```

#### Enter path

You can enter a normal path:

```text
/Users/reza/Movies/video.mp4
```

Home-directory shortcuts are supported:

```text
~/Movies/video.mp4
$HOME/Movies/video.mp4
```

Quoted paths are also accepted:

```text
"$HOME/Movies/video.mp4"
```

#### Browse

The browser lets you navigate directories and select a video file.

```text
OFFLINE BROWSER

/Users/reza/Downloads

0. ..
 1. Movies/
 2. video.mp4
 3. another-video.mkv

q. Cancel
```

#### Search

Search recursively from the current directory:

```text
Search: sons
```

Matching video files are then listed for selection.

---

# Command-Line Mode

The original CLI interface is still supported:

```bash
cargo run -- <source> <renderer>
```

For example:

```bash
cargo run -- video.mkv 4
```

The renderer numbers are:

| Renderer | Mode        |
| -------- | ----------- |
| `1`      | Retro ASCII |
| `2`      | Retro Color |
| `3`      | VHS         |
| `4`      | Video       |

---

# Renderers

All four renderers are wrapped inside the retro CRT television cabinet.

## 1. Retro ASCII

```bash
cargo run -- video.mkv 1
```

A monochrome renderer using block characters.

```text
████████████████████████████
██████▀▀▀▀▀▀▀▀▀▀▀▀██████████
████                  ██████
███                    █████
████                  ██████
██████▄▄▄▄▄▄▄▄▄▄▄▄██████████
████████████████████████████
```

Designed for:

- Low visual complexity
- Low terminal bandwidth
- Strong contrast
- Retro computer aesthetics

---

## 2. Retro Color

```bash
cargo run -- video.mkv 2
```

Uses ANSI colors and half-block characters.

A single terminal cell can represent two vertical pixels:

```text
▀
```

The foreground represents the upper pixel while the background represents the lower pixel.

This provides considerably more visual information than ordinary ASCII rendering while keeping the output relatively lightweight.

---

## 3. VHS

```bash
cargo run -- video.mkv 3
```

The VHS renderer builds on terminal color rendering and applies a deliberately degraded analog-video aesthetic.

The effect can include things such as:

- Scanlines
- Brightness variation
- Color imperfections
- Tracking-style distortion
- Analog noise
- Small visual instability

The effects are applied to the rendered output rather than modifying the original video.

---

## 4. Video

```bash
cargo run -- video.mkv 4
```

The highest-quality renderer.

It uses:

- RGB frames
- ANSI truecolor
- Half-block rendering
- Higher resolution
- Higher frame rate

It is still terminal video, so it is fundamentally limited by the terminal's dimensions and rendering performance.

It is not intended to compete with graphical players such as VLC, mpv, or QuickTime.

---

# Video Sources

## Local Files

Any supported local video file can be supplied directly:

```bash
cargo run -- "/Users/reza/Movies/video.mp4" 4
```

Supported formats currently include:

```text
mp4
mkv
webm
mov
m4v
avi
wmv
flv
mpg
mpeg
ts
mts
m2ts
3gp
ogv
```

FFmpeg ultimately handles the actual media decoding.

---

## Direct URLs

HTTP and HTTPS media URLs can be passed directly:

```bash
cargo run -- "https://example.com/video.mp4" 4
```

RTMP and RTSP URLs are also recognized by the source resolver:

```bash
cargo run -- "rtmp://example.com/live" 4
```

```bash
cargo run -- "rtsp://example.com/live" 4
```

Whether a particular stream can be played depends on FFmpeg's support for that source.

---

## YouTube

YouTube URLs are resolved through `yt-dlp`:

```bash
cargo run -- "https://www.youtube.com/watch?v=WvV5TbJc9tQ" 4
```

The flow is:

```text
YouTube URL
     │
     ▼
   yt-dlp
     │
     ▼
Media stream URL
     │
     ▼
   FFmpeg
     │
     ▼
 RGB frames
```

YouTube-specific logic remains isolated from the decoder and renderers.

---

# Decoder Profiles

Different rendering modes do not need the same amount of video data.

Retro modes use smaller frames and lower frame rates, while the Video renderer uses a larger profile.

Conceptually:

```text
Retro
 ├── smaller resolution
 └── lower FPS

VHS
 ├── smaller resolution
 └── lower FPS

Video
 ├── larger resolution
 └── higher FPS
```

This prevents the application from unnecessarily processing large source frames when the renderer cannot display them.

---

# CRT / Retro TV

Every renderer is currently wrapped in a reusable CRT television renderer.

Conceptually:

```text
┌─────────────────────────────────────────┐
│                                         │
│       ┌─────────────────────────┐       │
│       │                         │       │
│       │       VIDEO FRAME       │       │
│       │                         │       │
│       └─────────────────────────┘       │
│                                         │
│   ░▒▓░▒▓░▒▓░▒▓░▒▓░▒▓░▒▓░▒▓░▒▓░▒▓       │
└─────────────────────────────────────────┘
```

The TV wrapper does not know how the actual image is rendered.

It simply takes the output of another renderer and places it inside the cabinet.

This keeps the TV effect independent from the four rendering modes.

---

# Architecture

The project follows a small pipeline:

```text
Source → Decoder → Player → Renderer → Terminal
```

Current structure:

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
│   ├── offline.rs
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
│   ├── video.rs
│   └── tv.rs
│
├── player/
│   └── mod.rs
│
└── terminal/
    └── mod.rs
```

### Input

Handles:

- Interactive startup
- Online/offline selection
- Local path input
- Directory browsing
- Recursive search
- Renderer selection
- CLI arguments

### Source

Handles source identification and resolution:

```text
Local
Direct
YouTube
```

### Decoder

Starts FFmpeg and converts the source into raw RGB frames.

The decoder exposes frames as:

```rust
VideoFrame {
    width,
    height,
    pixels,
}
```

### Player

Controls the playback loop and frame timing.

```text
Decode
  ↓
Render
  ↓
Draw
  ↓
Wait
  ↓
Next frame
```

### Renderer

Converts RGB frames into terminal output.

Current renderers:

```text
AsciiRenderer
ColorRenderer
VhsRenderer
VideoRenderer
TvRenderer
```

`TvRenderer` acts as a wrapper around the other renderers.

### Terminal

Handles terminal-specific operations such as:

- Alternate screen
- Cursor visibility
- Screen clearing
- Cursor positioning
- ANSI output
- Terminal restoration

---

# Comparing the Renderers

The repository includes:

```text
compare.sh
```

Make it executable:

```bash
chmod +x compare.sh
```

Run:

```bash
./compare.sh
```

The script launches the four renderers using the same source so they can be visually compared.

```text
Terminal 1 → Retro ASCII
Terminal 2 → Retro Color
Terminal 3 → VHS
Terminal 4 → Video
```

This is primarily a **visual comparison tool**, not a performance benchmark.

Running four FFmpeg processes and four terminal renderers simultaneously naturally consumes considerably more CPU and resources.

---

# Performance

Terminal video is fundamentally different from graphical video playback.

For every frame, the application must:

```text
Decode video
    ↓
Scale frame
    ↓
Convert pixels
    ↓
Generate ANSI output
    ↓
Write to terminal
    ↓
Terminal parses and renders output
```

The main limitations are therefore:

- Terminal dimensions
- ANSI output size
- Terminal rendering speed
- Frame resolution
- Frame rate
- FFmpeg decoding cost
- Network speed for online sources

The decoder intentionally scales frames before rendering so that the application does not process unnecessary resolution.

---

# Dependencies

The Rust project currently has no runtime Cargo dependencies:

```toml
[dependencies]
```

Media decoding is delegated to external tools:

```text
FFmpeg
yt-dlp
```

This keeps the Rust side small while relying on mature media software for codec and container support.

---

# Testing

Format the project:

```bash
cargo fmt
```

Check formatting:

```bash
cargo fmt --check
```

Run the compiler checks:

```bash
cargo check
```

Run tests:

```bash
cargo test
```

---

# Current Limitations

RetroTermPlayer is still an experimental project.

Current limitations include:

- Terminal resolution limits visual quality
- ANSI rendering can become CPU-intensive at higher resolutions
- Terminal size affects the usable picture area
- FFmpeg must be installed separately
- `yt-dlp` is required for YouTube sources
- Network playback depends on connection stability
- Some media URLs may not be directly supported by FFmpeg
- Audio playback is not currently implemented
- Playback controls are not yet implemented
- Terminal truecolor support depends on the terminal emulator

---

# Audio

Audio is intentionally outside the current scope.

The current pipeline is:

```text
Source
  ↓
FFmpeg
  ↓
RGB Video Frames
  ↓
Renderer
  ↓
Terminal
```

There is currently no audio output or audio/video synchronization layer.

This project started as an experiment in terminal video rendering. Audio can be added later without requiring the renderers themselves to understand audio.

---

# PJ-PLAYER

RetroTermPlayer is also an exploration of a reusable terminal video engine for [PJ-PLAYER](https://github.com/rezkhaleghi/pj-player).

The long-term idea is to keep the responsibilities separate:

```text
PJ-PLAYER
   │
   ├── Audio playback
   │
   └── Optional terminal video
             │
             ▼
       RetroTermPlayer
             │
             ├── YouTube / media source
             ├── FFmpeg decoder
             └── Terminal renderer
```

RetroTermPlayer remains a standalone project while the rendering components are developed independently.

---

# Development Philosophy

RetroTermPlayer deliberately avoids unnecessary complexity.

The core rule is:

> **Each component should do one thing and know as little as possible about the others.**

The source layer should not know about rendering.

The renderer should not know where the video came from.

The player should not know how ANSI rendering works.

The terminal should not care whether the source is local, remote, or YouTube.

The goal is a small, reusable terminal media engine rather than another media framework.

---

# License

RetroTermPlayer is licensed under the MIT License.

See [LICENSE](LICENSE) for the full license text.
