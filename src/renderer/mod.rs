use crate::decoder::VideoFrame;

mod ascii;
mod ascii_shading;
mod color;
mod truecolor;
mod tv;
mod video;

pub use ascii::AsciiRenderer;
pub use ascii_shading::AsciiShadingRenderer;
pub use color::ColorRenderer;
pub use truecolor::TrueColorRenderer;
pub use tv::TvRenderer;
pub use video::VideoRenderer;

#[derive(Debug, Clone, Copy)]
pub enum RendererKind {
    Ascii,
    MonoBlock,
    ColorBlock,
    Video,
    TrueColor,
}

pub trait Renderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String);
}

pub fn create_renderer(kind: RendererKind) -> Box<dyn Renderer> {
    match kind {
        RendererKind::Ascii => Box::new(AsciiShadingRenderer::new()),
        RendererKind::MonoBlock => Box::new(AsciiRenderer::new()),
        RendererKind::ColorBlock => Box::new(ColorRenderer::new()),
        RendererKind::Video => Box::new(VideoRenderer::new()),
        RendererKind::TrueColor => Box::new(TrueColorRenderer::new()),
    }
}
