use core::gba::GameBoyAdvance;
use std::{fs::File, io::Read};

use clap::{Arg, ArgAction, Command};

fn main() {
    let arg_matches = Command::new("Iron Boy Advance")
        .about("CLI for Iron Boy Advance")
        .arg(Arg::new("rom").short('r').long("rom").required(true).help("Rom file to be loaded"))
        .arg(Arg::new("bios").short('b').long("bios").help("Bios file to be loaded"))
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

    let rom_path = arg_matches.get_one::<String>("rom").expect("Rom is required");
    let mut rom = File::open(rom_path).expect("Unable to open rom file");
    let mut rom_buffer = Vec::new();
    rom.read_to_end(&mut rom_buffer).expect("Issue while reading rom file");

    let bios_path = arg_matches.get_one::<String>("bios");
    let bios_buffer = match bios_path {
        Some(path) => {
            let mut bios = File::open(path).expect("Unable to open bios file");
            let mut buffer = Vec::new();
            bios.read_to_end(&mut buffer).expect("Issue while reading bios file");
            Some(buffer)
        }
        None => None,
    };

    let show_logs = arg_matches.get_flag("logs");
    let game_boy_advance = GameBoyAdvance::new(rom_buffer, bios_buffer, show_logs);

    let _show_memory = arg_matches.get_flag("memory");
    let _show_vram = arg_matches.get_flag("vram");

    //todo build out the windows
}
