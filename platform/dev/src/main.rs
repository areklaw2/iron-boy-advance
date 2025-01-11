use core::gba::GameBoyAdvance;

use clap::{Arg, ArgAction, Command};

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
    let rom_path = arg_matches
        .get_one::<String>("rom")
        .expect("Rom is required");

    let bios_path = arg_matches
        .get_one::<String>("bios")
        .expect("Bios is required");

    let show_logs = arg_matches.get_flag("logs");
    let game_boy_advance =
        GameBoyAdvance::new(rom_path.into(), bios_path.into(), show_logs).unwrap();

    let _show_memory = arg_matches.get_flag("memory");
    let _show_vram = arg_matches.get_flag("vram");

    //todo build out the windows
}
