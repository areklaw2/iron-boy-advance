use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ppu::CYCLES_PER_FRAME;

mod bios;
mod cartridge;
pub mod gba;
mod interrupt_control;
mod io_registers;
mod ppu;
mod scheduler;
mod system_bus;
mod system_control;

pub const FPS: f32 = CPU_CLOCK_SPEED as f32 / CYCLES_PER_FRAME as f32;
