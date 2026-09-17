//! Core library for RetroTermPlayer.
//!
//! The project is intentionally split into independent modules so that
//! the video source, decoder, terminal handling, and renderers can be
//! reused by other Rust applications such as PJ-PLAYER.

pub mod decoder;
pub mod input;
pub mod player;
pub mod renderer;
pub mod source;
pub mod terminal;
