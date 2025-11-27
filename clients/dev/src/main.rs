use std::fs::OpenOptions;

use ironboyadvance_core::{FPS, GameBoyAdvance};

use clap::{ArgAction, Parser};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const FRAME_DURATION_NANOS: f32 = 1_000_000_000.0 / FPS;
const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(FRAME_DURATION_NANOS as u64);

#[derive(Parser)]
#[command(name = "Iron Boy Advance")]
#[command(about = "CLI for Iron Boy Advance", long_about = None)]
struct DeveloperCli {
    #[arg(short, long, help = "Rom file to be loaded")]
    rom: String,
    #[arg(short, long, help = "Bios file to be loaded")]
    bios: String,
    #[arg(short, long, action = ArgAction::SetTrue, required = false, help = "Skips bios")]
    skip_bios: bool,
    #[arg(short, long, action = ArgAction::SetTrue, required = false, help = "Opens log viewer window")]
    logs: bool,
    #[arg(short, long, action = ArgAction::SetTrue, required = false, help = "Opens memory viewer window")]
    memory: bool,
    #[arg(short, long, action = ArgAction::SetTrue, required = false, help = "Opens vram viewer window")]
    vram: bool,
}

fn main() {
    let cli = DeveloperCli::parse();
    let show_logs = cli.logs;
    let _show_memory = cli.memory;
    let _show_vram = cli.vram;
    initilize_logger();

    //TODO: build out the windows
    let mut game_boy_advance = GameBoyAdvance::new(cli.rom.into(), cli.bios.into(), show_logs, cli.skip_bios).unwrap();
    let mut overshoot = 0;
    'game: loop {
        let frame_start_time = std::time::Instant::now();
        overshoot = game_boy_advance.run(overshoot);

        while frame_start_time.elapsed().as_micros() < FRAME_DURATION.as_micros() {
            std::hint::spin_loop();
        }
    }
}

fn initilize_logger() {
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("ironboyadvance.log")
        .expect("Failed to create log file");

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(log_file)
                .with_ansi(false)
                .without_time() // remove this
                .with_target(false) // remove this
                .with_level(false) // remove this
                .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))),
        )
        .init();
}
