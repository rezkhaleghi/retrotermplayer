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
/// The renderer writes into a caller-owned String instead of returning a
/// newly allocated String for every frame. The player reuses that buffer
/// throughout playback, which significantly reduces allocation churn.
pub trait Renderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String);
}

/// Creates the renderer associated with the selected visual mode.
pub fn create_renderer(kind: RendererKind) -> Box<dyn Renderer> {
    match kind {
        RendererKind::Ascii => Box::new(AsciiRenderer::new()),
        RendererKind::Color => Box::new(ColorRenderer::new()),
        RendererKind::Vhs => Box::new(VhsRenderer::new()),
        RendererKind::Video => Box::new(VideoRenderer::new()),
    }
}
