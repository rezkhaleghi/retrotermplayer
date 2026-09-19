use retrotermplayer::{
    decoder::{DecoderProfile, FfmpegDecoder},
    input::read_input,
    player::Player,
    renderer::{create_renderer, Renderer, RendererKind, TvRenderer},
    source::{resolve_source, VideoQuality},
    terminal::Terminal,
};

fn main() {
    let input = match read_input() {
        Ok(input) => input,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::Interrupted {
                eprintln!("{error}");
            }

            return;
        }
    };

    let source = match resolve_source(&input.source) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("\nUnable to resolve video source:\n{error}");
            return;
        }
    };

    println!("Source: {}", source.description());

    let (renderer_kind, decoder_profile, video_quality) = match input.renderer {
        1 => (
            RendererKind::Ascii,
            DecoderProfile::RETRO,
            VideoQuality::Low,
        ),
        2 => (
            RendererKind::MonoBlock,
            DecoderProfile::RETRO,
            VideoQuality::Low,
        ),
        3 => (
            RendererKind::MonoVideo,
            DecoderProfile::MONO_VIDEO,
            VideoQuality::Normal,
        ),
        4 => (
            RendererKind::Video,
            DecoderProfile::VIDEO,
            VideoQuality::High,
        ),
        _ => unreachable!(),
    };

    // Create the actual visual renderer first.
    let renderer = create_renderer(renderer_kind);

    // Every visual mode is displayed inside the same CRT television.
    let renderer: Box<dyn Renderer> = Box::new(TvRenderer::new(renderer));

    let decoder = match FfmpegDecoder::new(source, decoder_profile, video_quality) {
        Ok(decoder) => decoder,
        Err(error) => {
            eprintln!("Failed to start FFmpeg:\n{error}");
            return;
        }
    };

    let terminal = Terminal::new();

    let mut player = Player::new(decoder, renderer, terminal);

    if let Err(error) = player.play() {
        eprintln!("\nPlayback error: {error}");
    }
}
