use clap::{arg, Arg, ArgAction, Command};

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

    let rom = arg_matches
        .get_one::<String>("rom")
        .expect("Rom is required");
    let bios = arg_matches.get_one::<String>("bios");
    let show_log_viewer = arg_matches.get_flag("logs");
    let show_memory_viewer = arg_matches.get_flag("memory");
    let show_vram_viewer = arg_matches.get_flag("vram");
}
