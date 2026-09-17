use retrotermplayer::{
    decoder::{DecoderProfile, FfmpegDecoder},
    input::read_input,
    player::Player,
    renderer::{create_renderer, RendererKind},
    source::{resolve_source, VideoQuality},
    terminal::Terminal,
};

fn main() {
    let input = match read_input() {
        Ok(input) => input,
        Err(error) => {
            eprintln!("{error}");
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
        // ASCII intentionally uses the lowest source quality.
        1 => (
            RendererKind::Ascii,
            DecoderProfile::RETRO,
            VideoQuality::Low,
        ),

        // Color also works well with a low-quality source because the
        // renderer already reduces the image to terminal resolution.
        2 => (
            RendererKind::Color,
            DecoderProfile::RETRO,
            VideoQuality::Low,
        ),

        // VHS benefits from a little more source detail before applying
        // its intentional degradation.
        3 => (RendererKind::Vhs, DecoderProfile::VHS, VideoQuality::Medium),

        // Normal Video mode gets the highest source quality we need.
        4 => (
            RendererKind::Video,
            DecoderProfile::VIDEO,
            VideoQuality::High,
        ),

        _ => unreachable!(),
    };

    let renderer = create_renderer(renderer_kind);

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
