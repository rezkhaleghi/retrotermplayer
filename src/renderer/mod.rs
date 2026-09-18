use crate::decoder::VideoFrame;

mod ascii;
mod ascii_shading;
mod color;
mod tv;
mod vhs;
mod video;

pub use ascii::AsciiRenderer;
pub use ascii_shading::AsciiShadingRenderer;
pub use color::ColorRenderer;
pub use tv::TvRenderer;
pub use vhs::VhsRenderer;
pub use video::VideoRenderer;

/// Available visual modes.
#[derive(Debug, Clone, Copy)]
pub enum RendererKind {
    AsciiShading,
    Ascii,
    Color,
    Vhs,
    Video,
}

/// Converts decoded video frames into terminal output.
pub trait Renderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String);
}

/// Creates the renderer associated with the selected visual mode.
pub fn create_renderer(kind: RendererKind) -> Box<dyn Renderer> {
    match kind {
        RendererKind::AsciiShading => Box::new(AsciiShadingRenderer::new()),
        RendererKind::Ascii => Box::new(AsciiRenderer::new()),
        RendererKind::Color => Box::new(ColorRenderer::new()),
        RendererKind::Vhs => Box::new(VhsRenderer::new()),
        RendererKind::Video => Box::new(VideoRenderer::new()),
    }
}
