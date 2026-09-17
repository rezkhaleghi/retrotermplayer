use retrotermplayer::{
    decoder::{DecoderProfile, FfmpegDecoder},
    input::read_input,
    player::Player,
    renderer::{create_renderer, RendererKind},
    source::resolve_source,
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

    let (renderer_kind, decoder_profile) = match input.renderer {
        1 => (RendererKind::Ascii, DecoderProfile::RETRO),
        2 => (RendererKind::Color, DecoderProfile::RETRO),
        3 => (RendererKind::Vhs, DecoderProfile::VHS),
        4 => (RendererKind::Video, DecoderProfile::VIDEO),
        _ => unreachable!(),
    };

    let renderer = create_renderer(renderer_kind);

    let decoder = match FfmpegDecoder::new(source, decoder_profile) {
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
