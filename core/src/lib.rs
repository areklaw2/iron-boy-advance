use core::error;
use std::{fs::File, io::Read, path::PathBuf};

use thiserror::Error;

mod arm7tdmi;
mod bios;
mod cartridge;
pub mod gba;
mod memory;
pub mod ppu;
mod scheduler;
pub mod sharp_sm83;

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
