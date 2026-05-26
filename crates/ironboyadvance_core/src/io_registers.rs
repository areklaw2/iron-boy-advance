use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters, Setters};
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps, scheduler::Scheduler};
use tracing::debug;

use crate::{
    dma_control::DmaController, events::GbaEvent, interrupt_control::InterruptController, keypad::Keypad, ppu::Ppu,
    system_control::SystemController, timer_control::TimerController,
};

#[derive(Getters, MutGetters, Setters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct IoRegisters {
    ppu: Ppu,
    dma_controller: DmaController,
    timer_controller: TimerController,
    // TODO remove when doing sound this just gets the bios to pass
    sound_bias: u16,
    keypad: Keypad,
    interrupt_controller: InterruptController,
    system_controller: SystemController,
}

impl IoRegisters {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        IoRegisters {
            ppu: Ppu::new(),
            dma_controller: DmaController::new(scheduler.clone()),
            timer_controller: TimerController::new(scheduler),
            keypad: Keypad::new(),
            interrupt_controller: InterruptController::new(),
            system_controller: SystemController::new(),
            sound_bias: 0x0200,
        }
    }
}

impl SystemMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // PPU
            0x04000000..=0x04000057 => self.ppu.read_8(address),
            // TODO remove when doing sound this just gets the bios to pass
            0x04000088..=0x04000089 => self.sound_bias.read_byte(address),
            // DMA Control
            0x040000B0..=0x040000DF => self.dma_controller.read_8(address),
            // Timer Control
            0x04000100..=0x0400010F => self.timer_controller.read_8(address),
            // Keypad
            0x04000130..=0x04000133 => self.keypad.read_8(address),
            // Interrupt Control
            0x04000200..=0x04000203 | 0x04000208..=0x0400020B => self.interrupt_controller.read_8(address),
            // System Control
            0x04000204..=0x04000207 | 0x04000300..=0x04000301 | 0x04000410 => self.system_controller.read_8(address),
            0x04000800..=0x04FFFFFF => self.system_controller.read_8(address), // Mirroring for 0x04000800
            // Access Memory
            0x05000000..=0x05FFFFFF => self.ppu.read_8(address),
            0x06000000..=0x06FFFFFF => self.ppu.read_8(address),
            0x07000000..=0x07FFFFFF => self.ppu.read_8(address),
            _ => {
                debug!("Read byte not implemented for I/O register: {:#010X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // PPU
            0x04000000..=0x04000057 => self.ppu.write_8(address, value),
            // TODO remove when doing sound this just gets the bios to pass
            0x04000088..=0x04000089 => self.sound_bias.write_byte(address, value),
            // DMA Control
            0x040000B0..=0x040000DF => self.dma_controller.write_8(address, value),
            // Timer Control
            0x04000100..=0x0400010F => self.timer_controller.write_8(address, value),
            // Keypad
            0x04000130..=0x04000133 => self.keypad.write_8(address, value),
            // Interrupt Control
            0x04000200..=0x04000203 | 0x04000208..=0x0400020B => self.interrupt_controller.write_8(address, value),
            // System Control
            0x04000204..=0x04000207 | 0x04000300..=0x04000301 | 0x04000410 => self.system_controller.write_8(address, value),
            0x04000800..=0x04FFFFFF => self.system_controller.write_8(address, value), // Mirroring for 0x04000800
            // Access Memory
            0x05000000..=0x05FFFFFF => self.ppu.write_8(address, value),
            0x06000000..=0x06FFFFFF => self.ppu.write_8(address, value),
            0x07000000..=0x07FFFFFF => self.ppu.write_8(address, value),
            _ => debug!(
                "Write byte not implemented for I/O register: {:#010X}, value: {:#04X}",
                address, value
            ),
        }
    }
}
