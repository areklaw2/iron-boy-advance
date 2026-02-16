use dev::Application;
use clap::{ArgAction, Parser};

#[derive(Parser)]
#[command(name = "Iron Boy Advance")]
struct DeveloperCli {
    #[arg(short, long)]
    rom: String,
    #[arg(short, long)]
    bios: String,
    #[arg(short, long, action = ArgAction::SetTrue, required = false)]
    skip_bios: bool,
    #[arg(short, long, action = ArgAction::SetTrue, required = false)]
    logs: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = DeveloperCli::parse();
    let mut application = Application::new(cli.rom, cli.bios, cli.skip_bios, cli.logs)?;
    application.run()?;
    Ok(())
}
