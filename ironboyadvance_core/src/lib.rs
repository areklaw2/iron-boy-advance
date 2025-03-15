use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ppu::CYCLES_PER_FRAME;
use thiserror::Error;

mod bios;
mod cartridge;
pub mod gba;
pub mod ppu;
mod scheduler;
mod system_bus;

pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;

#[derive(Error, Debug)]
pub enum GbaError {
    #[error("Unable to open file")]
    FileLoadFailure,
    #[error("Cartridge checksum invalid")]
    CartridgeCheckSumFailure,
    #[error("Header length incorrect")]
    IncorrectHeaderLength,
    #[error("Header parsing failed")]
    HeaderParseFailure,
}
