use retrotermplayer::{
    decoder::{probe_duration, DecoderProfile, FfmpegDecoder},
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

    // Resolve the source once. YouTube sources become a direct media URL here.
    // The decoder reuses that resolved input when seeking.
    let resolved_input = match source.resolve_for_ffmpeg(video_quality) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("Failed to resolve media stream:\n{error}");
            return;
        }
    };

    let duration = match probe_duration(&resolved_input) {
        Ok(duration) => duration,
        Err(error) => {
            eprintln!("Warning: could not determine video duration: {error}");
            None
        }
    };

    if let Some(duration) = duration {
        println!("Duration: {:.0}s", duration);
    }

    // Create the actual visual renderer first.
    let renderer = create_renderer(renderer_kind);

    // Every visual mode is displayed inside the same CRT television.
    let renderer: Box<dyn Renderer> = Box::new(TvRenderer::new(renderer));

    let decoder = match FfmpegDecoder::new_from_input(
        resolved_input,
        decoder_profile,
        video_quality,
        0.0,
    ) {
        Ok(decoder) => decoder,
        Err(error) => {
            eprintln!("Failed to start FFmpeg:\n{error}");
            return;
        }
    };

    let terminal = Terminal::new();

    let mut player = Player::new(decoder, renderer, terminal, duration);

    if let Err(error) = player.play() {
        eprintln!("\nPlayback error: {error}");
    }
}