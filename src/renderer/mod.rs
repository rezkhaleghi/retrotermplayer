use crate::decoder::VideoFrame;

mod ascii;
mod ascii_shading;
mod mono_video;
mod tv;
mod video;

pub use ascii::AsciiRenderer;
pub use ascii_shading::AsciiShadingRenderer;
pub use mono_video::MonoVideoRenderer;
pub use tv::TvRenderer;
pub use video::VideoRenderer;

#[derive(Debug, Clone, Copy)]
pub enum RendererKind {
    Ascii,
    MonoBlock,
    MonoVideo,
    Video,
}

pub trait Renderer {
    fn render(&mut self, frame: &VideoFrame, output: &mut String);
}

pub fn create_renderer(kind: RendererKind) -> Box<dyn Renderer> {
    match kind {
        RendererKind::Ascii => Box::new(AsciiShadingRenderer::new()),

        RendererKind::MonoBlock => Box::new(AsciiRenderer::new()),

        RendererKind::MonoVideo => Box::new(MonoVideoRenderer::new()),

        RendererKind::Video => Box::new(VideoRenderer::new()),
    }
}
