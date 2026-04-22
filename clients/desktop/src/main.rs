use clap::{ArgAction, Parser};

#[derive(Parser)]
#[command(name = "Iron Boy Advance")]
struct DesktopCli {
    #[arg(short, long)]
    rom: String,
    #[arg(short, long, required = false)]
    bios: Option<String>,
    #[arg(short, long, action = ArgAction::SetTrue, required = false)]
    logs: bool,
}

fn main() -> Result<(), desktop::DesktopError> {
    let cli = DesktopCli::parse();
    desktop::run(cli.rom, cli.bios, cli.logs)?;
    Ok(())
}
