use crate::decoder::VideoFrame;

mod ascii;
mod color;
mod vhs;
mod video;

pub use ascii::AsciiRenderer;
pub use color::ColorRenderer;
pub use vhs::VhsRenderer;
pub use video::VideoRenderer;

/// Available visual modes.
///
/// Each mode represents a genuinely different presentation of the decoded
/// video rather than simply another alias for an existing renderer.
#[derive(Debug, Clone, Copy)]
pub enum RendererKind {
    Ascii,
    Color,
    Vhs,
    Video,
}

/// Converts decoded video frames into terminal output.
///
/// Renderers know nothing about where the video came from. They receive
/// the same VideoFrame regardless of whether the source is YouTube,
/// a local file, or a direct media URL.
pub trait Renderer {
    fn render(&mut self, frame: &VideoFrame) -> String;
}

pub fn create_renderer(kind: RendererKind) -> Box<dyn Renderer> {
    match kind {
        RendererKind::Ascii => Box::new(AsciiRenderer::new()),
        RendererKind::Color => Box::new(ColorRenderer::new()),
        RendererKind::Vhs => Box::new(VhsRenderer::new()),
        RendererKind::Video => Box::new(VideoRenderer::new()),
    }
}