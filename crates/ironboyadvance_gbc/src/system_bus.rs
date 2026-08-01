use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters};
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::{
    GbSpeed,
    memory::{InterruptContext, MemoryInterface},
};
use tracing::debug;

use crate::{
    boot_rom::BootRom,
    cartridge::Cartridge,
    events::{GbcEvent, InterruptEvent, SerialEvent},
    io_registers::IoRegisters,
    memory::Memory,
};

const NORMAL_SPEED_M_CYCLES: usize = 4;
const DOUBLE_SPEED_M_CYCLES: usize = 2;

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct SystemBus {
    boot_rom: BootRom,
    cartridge: Cartridge,
    memory: Memory,
    io_registers: IoRegisters,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl SystemBus {
    pub fn new(cartridge: Cartridge, boot_rom: BootRom, scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        SystemBus {
            boot_rom,
            cartridge,
            memory: Memory::new(),
            io_registers: IoRegisters::new(scheduler.clone()),
            scheduler,
        }
    }

    fn m_cycle(&self) {
        let ticks = match self.speed() {
            GbSpeed::Normal => NORMAL_SPEED_M_CYCLES,
            GbSpeed::Double => DOUBLE_SPEED_M_CYCLES,
        };

        self.scheduler.borrow_mut().step(ticks);
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) {
        self.io_registers.interrupt_controller_mut().raise_interrupt(interrupt_event);
    }

    pub fn interrupts_pending(&self) -> bool {
        self.io_registers.interrupt_controller().interrupts_pending()
    }

    pub fn handle_serial_event(&mut self, serial_event: SerialEvent, timestamp: usize) {
        self.io_registers.serial_transfer_mut().handle_event(serial_event, timestamp);
    }

    pub fn speed(&self) -> GbSpeed {
        self.io_registers.speed_controller().speed()
    }
}

impl SystemMemoryAccess for SystemBus {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            _ if self.boot_rom.contains(address) => self.boot_rom.read_8(address),
            0xFF50 => self.boot_rom.read_8(address),
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read_8(address),
            0xC000..=0xFDFF | 0xFF70 | 0xFF80..=0xFFFE => self.memory.read_8(address),
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF00..=0xFF7F | 0xFFFF => self.io_registers.read_8(address),
            _ => {
                debug!("Read byte not implemented for address: {:#06X}", address);
                0xFF
            }
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF50 => self.boot_rom.write_8(address, value),
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write_8(address, value),
            0xC000..=0xFDFF | 0xFF70 | 0xFF80..=0xFFFE => self.memory.write_8(address, value),
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF00..=0xFF7F | 0xFFFF => self.io_registers.write_8(address, value),
            _ => debug!("Write byte not implemented for address: {:#06X}", address),
        }
    }
}

impl MemoryInterface for SystemBus {
    fn load_8(&self, address: u16) -> u8 {
        self.m_cycle();
        self.read_8(address)
    }

    fn load_16(&self, address: u16) -> u16 {
        let low = self.load_8(address) as u16;
        let high = self.load_8(address.wrapping_add(1)) as u16;
        high << 8 | low
    }

    fn store_8(&mut self, address: u16, value: u8) {
        self.m_cycle();
        self.write_8(address, value);
    }

    fn store_16(&mut self, address: u16, value: u16) {
        self.store_8(address, value as u8);
        self.store_8(address.wrapping_add(1), (value >> 8) as u8);
    }

    fn idle_cycle(&mut self) {
        self.m_cycle();
    }

    fn change_speed(&mut self) {
        self.io_registers.speed_controller_mut().change_speed();
        let speed = self.speed();
        self.io_registers.serial_transfer_mut().set_speed(speed);
    }

    fn interrupt_context(&self) -> &InterruptContext {
        self.io_registers.interrupt_controller().interrupt_context()
    }

    fn interrupt_context_mut(&mut self) -> &mut InterruptContext {
        self.io_registers.interrupt_controller_mut().interrupt_context_mut()
    }
}
