use core::{gba::GameBoyAdvance, FPS};

use clap::{ArgAction, Parser};

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

    //TODO: build out the windows
    let mut game_boy_advance = GameBoyAdvance::new(cli.rom.into(), cli.bios.into(), show_logs).unwrap();
    let mut overshoot = 0;
    'game: loop {
        let frame_start_time = std::time::Instant::now();
        overshoot = game_boy_advance.run(overshoot);

        while frame_start_time.elapsed().as_micros() < FRAME_DURATION.as_micros() {
            std::hint::spin_loop();
        }
    }
}
