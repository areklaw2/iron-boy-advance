use core::{gba::GameBoyAdvance, FPS};

use clap::{Arg, ArgAction, Command};

const FRAME_DURATION_NANOS: f32 = 1_000_000_000.0 / FPS;
const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos(FRAME_DURATION_NANOS as u64);

fn main() {
    let arg_matches = Command::new("Iron Boy Advance")
        .about("CLI for Iron Boy Advance")
        .arg(
            Arg::new("rom")
                .short('r')
                .long("rom")
                .required(true)
                .help("Rom file to be loaded"),
        )
        .arg(
            Arg::new("bios")
                .short('b')
                .long("bios")
                .required(true)
                .help("Bios file to be loaded"),
        )
        .arg(
            Arg::new("logs")
                .short('l')
                .long("logs")
                .help("Opens log viewer window")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("memory")
                .short('m')
                .long("memory")
                .help("Opens memory viewer window")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("vram")
                .short('v')
                .long("vram")
                .help("Opens vram viewer window")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    //turn these into pathbufs
    let rom_path = arg_matches.get_one::<String>("rom").expect("Rom is required");
    let bios_path = arg_matches.get_one::<String>("bios").expect("Bios is required");

    let show_logs = arg_matches.get_flag("logs");

    let _show_memory = arg_matches.get_flag("memory");
    let _show_vram = arg_matches.get_flag("vram");

    //TODO: build out the windows
    let mut game_boy_advance = GameBoyAdvance::new(rom_path.into(), bios_path.into(), show_logs).unwrap();
    let mut overshoot = 0;
    'game: loop {
        let frame_start_time = std::time::Instant::now();
        overshoot = game_boy_advance.run(overshoot);

        while frame_start_time.elapsed().as_micros() < FRAME_DURATION.as_micros() {
            std::hint::spin_loop();
        }
    }
}
