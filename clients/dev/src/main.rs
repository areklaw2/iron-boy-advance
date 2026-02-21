use clap::{ArgAction, Parser};
use dev::Application;

#[derive(Parser)]
#[command(name = "Iron Boy Advance")]
struct DeveloperCli {
    #[arg(short, long)]
    rom: String,
    #[arg(short, long, required = false)]
    bios: Option<String>,
    #[arg(short, long, action = ArgAction::SetTrue, required = false)]
    logs: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = DeveloperCli::parse();
    let mut application = Application::new(cli.rom, cli.bios, cli.logs)?;
    application.run()?;
    Ok(())
}
