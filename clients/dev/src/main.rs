use clap::{ArgAction, Parser};

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
    dev::run(cli.rom, cli.bios, cli.logs)?;
    Ok(())
}
