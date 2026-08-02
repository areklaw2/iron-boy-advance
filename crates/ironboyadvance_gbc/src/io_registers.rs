use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters};
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::GbMode;
use tracing::debug;

use crate::{
    apu::Apu,
    dma_control::DmaController,
    events::GbcEvent,
    interrupt_control::InterruptController,
    joypad::Joypad,
    ppu::{Ppu, registers::PpuMode},
    serial_transfer::SerialTransfer,
    speed_control::SpeedController,
    timer::Timer,
};

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct IoRegisters {
    ppu: Ppu,
    apu: Apu,
    joypad: Joypad,
    interrupt_controller: InterruptController,
    serial_transfer: SerialTransfer,
    speed_controller: SpeedController,
    timer: Timer,
    dma_controller: DmaController,
}

impl IoRegisters {
    pub fn new(mode: GbMode, skip_boot: bool, scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        IoRegisters {
            ppu: Ppu::new(mode, skip_boot, scheduler.clone()),
            apu: Apu::new(mode, scheduler.clone()),
            joypad: Joypad::new(),
            interrupt_controller: InterruptController::new(),
            serial_transfer: SerialTransfer::new(scheduler.clone()),
            speed_controller: SpeedController::new(),
            timer: Timer::new(scheduler.clone()),
            dma_controller: DmaController::new(scheduler),
        }
    }
}

impl SystemMemoryAccess for IoRegisters {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            // Joypad
            0xFF00 => self.joypad.read_8(address),
            // Serial Transfer
            0xFF01..=0xFF02 => self.serial_transfer.read_8(address),
            // Timer
            0xFF04..=0xFF07 => self.timer.read_8(address),
            // Dma
            0xFF46 | 0xFF51..=0xFF55 => self.dma_controller.read_8(address),
            // Interrupt Control
            0xFF0F | 0xFFFF => self.interrupt_controller.read_8(address),
            // Apu
            0xFF10..=0xFF3F => self.apu.read_8(address),
            // Ppu
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF45 | 0xFF47..=0xFF4C | 0xFF4E..=0xFF4F | 0xFF68..=0xFF6B => {
                self.ppu.read_8(address)
            }
            // Speed Control
            0xFF4D => self.speed_controller.read_8(address),
            _ => {
                debug!("Read byte not implemented for I/O register: {:#06X}", address);
                0xFF
            }
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            // Joypad
            0xFF00 => self.joypad.write_8(address, value),
            // Serial Transfer
            0xFF01..=0xFF02 => self.serial_transfer.write_8(address, value),
            // Timer
            0xFF04..=0xFF07 => self.timer.write_8(address, value),
            // Dma
            0xFF46 | 0xFF51..=0xFF54 => self.dma_controller.write_8(address, value),
            0xFF55 => {
                let in_h_blank = self.ppu.mode() == PpuMode::HBlank;
                self.dma_controller.write_vram_dma_control(value, in_h_blank);
            }
            // Interrupt Control
            0xFF0F | 0xFFFF => self.interrupt_controller.write_8(address, value),
            // Apu
            0xFF10..=0xFF3F => self.apu.write_8(address, value),
            // Ppu
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF45 | 0xFF47..=0xFF4C | 0xFF4E..=0xFF4F | 0xFF68..=0xFF6B => {
                self.ppu.write_8(address, value)
            }
            // Speed Control
            0xFF4D => self.speed_controller.write_8(address, value),
            _ => debug!("Write byte not implemented for I/O register: {:#06X}", address),
        }
    }
}
