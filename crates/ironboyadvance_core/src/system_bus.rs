use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters};
use ironboyadvance_arm7tdmi::memory::{MemoryAccessWidth, MemoryInterface, SystemMemoryAccess, decompose_access_pattern};
use tracing::debug;

use crate::{
    bios::Bios,
    cartridge::Cartridge,
    io_registers::IoRegisters,
    memory::Memory,
    ppu::HDRAW_CYCLES,
    scheduler::{
        Scheduler,
        event::{EventType, FutureEvent, InterruptEvent, PpuEvent},
    },
    system_control::HaltMode,
};

pub const BIOS_BASE: u32 = 0x0000_0000;
pub const WRAM_BOARD_BASE: u32 = 0x0200_0000;
pub const WRAM_CHIP_BASE: u32 = 0x0300_0000;
pub const IO_REGISTERS_BASE: u32 = 0x0400_0000;
pub const PALETTE_RAM_BASE: u32 = 0x0500_0000;
pub const VRAM_BASE: u32 = 0x0600_0000;
pub const OAM_BASE: u32 = 0x0700_0000;
pub const ROM_WS0_LO: u32 = 0x0800_0000;
pub const ROM_WS0_HI: u32 = 0x0900_0000;
pub const ROM_WS1_LO: u32 = 0x0A00_0000;
pub const ROM_WS1_HI: u32 = 0x0B00_0000;
pub const ROM_WS2_LO: u32 = 0x0C00_0000;
pub const ROM_WS2_HI: u32 = 0x0D00_0000;
pub const SRAM_LO: u32 = 0x0E00_0000;
pub const SRAM_HI: u32 = 0x0F00_0000;

#[derive(Getters, MutGetters)]
pub struct SystemBus {
    bios: Bios,
    memory: Memory,
    #[getset(get = "pub", get_mut = "pub")]
    io_registers: IoRegisters,
    cartridge: Cartridge,
    scheduler: Rc<RefCell<Scheduler>>,
}

impl MemoryInterface for SystemBus {
    fn load_8(&mut self, address: u32, access: u8) -> u32 {
        self.cycle(address, access, MemoryAccessWidth::Byte);
        self.read_8(address) as u32
    }

    fn load_16(&mut self, address: u32, access: u8) -> u32 {
        self.cycle(address, access, MemoryAccessWidth::HalfWord);
        self.read_16(address) as u32
    }

    fn load_32(&mut self, address: u32, access: u8) -> u32 {
        self.cycle(address, access, MemoryAccessWidth::Word);
        self.read_32(address)
    }

    fn store_8(&mut self, address: u32, value: u8, access: u8) {
        self.cycle(address, access, MemoryAccessWidth::Byte);
        self.write_8(address, value);
    }

    fn store_16(&mut self, address: u32, value: u16, access: u8) {
        self.cycle(address, access, MemoryAccessWidth::HalfWord);
        self.write_16(address, value);
    }

    fn store_32(&mut self, address: u32, value: u32, access: u8) {
        self.cycle(address, access, MemoryAccessWidth::Word);
        self.write_32(address, value);
    }

    fn idle_cycle(&mut self) {
        self.scheduler.borrow_mut().step(1);
    }
}

impl SystemMemoryAccess for SystemBus {
    fn read_8(&self, address: u32) -> u8 {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_8(address),
            WRAM_BOARD_BASE => self.memory.read_8(address),
            WRAM_CHIP_BASE => self.memory.read_8(address),
            IO_REGISTERS_BASE => self.io_registers.read_8(address), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.io_registers.read_8(address),
            VRAM_BASE => self.io_registers.read_8(address),
            OAM_BASE => self.io_registers.read_8(address),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_8(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_8(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_8(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_8(address),
            _ => {
                debug!("Unused: {:08X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_8(address, value),
            WRAM_BOARD_BASE => self.memory.write_8(address, value),
            WRAM_CHIP_BASE => self.memory.write_8(address, value),
            IO_REGISTERS_BASE => self.io_registers.write_8(address, value), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.io_registers.write_8(address, value),
            VRAM_BASE => self.io_registers.write_8(address, value),
            OAM_BASE => self.io_registers.write_8(address, value),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_8(address, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_8(address, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_8(address, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_8(address, value),
            _ => panic!("Unused: {:08X}", address),
        }
    }
}

impl SystemBus {
    pub fn new(cartridge: Cartridge, bios: Bios, scheduler: Rc<RefCell<Scheduler>>) -> Self {
        scheduler
            .borrow_mut()
            .schedule((EventType::Ppu(PpuEvent::HDraw), HDRAW_CYCLES));

        SystemBus {
            bios,
            memory: Memory::new(),
            io_registers: IoRegisters::new(),
            cartridge,
            scheduler,
        }
    }

    pub fn cycle(&mut self, address: u32, access_pattern: u8, width: MemoryAccessWidth) {
        let access = decompose_access_pattern(access_pattern)[0];
        let index = ((address >> 24) & 0xF) as usize;
        let cycles = self.io_registers.system_controller().cycles(index, width, access);
        self.scheduler.borrow_mut().step(cycles);
    }

    pub fn interrupt_pending(&self) -> bool {
        self.io_registers.interrupt_controller().interrupt_pending()
    }

    pub fn raise_interrupt(&mut self, interrupt_event: InterruptEvent) -> Vec<FutureEvent> {
        self.io_registers.interrupt_controller_mut().raise_interrupt(interrupt_event);
        vec![] // returning empty vec to satisfy caller
    }

    pub fn halt_mode(&self) -> HaltMode {
        self.io_registers.system_controller().halt_mode()
    }

    pub fn un_halt(&mut self) {
        self.io_registers.system_controller_mut().set_halt_mode(HaltMode::Running);
    }

    pub fn handle_ppu_event(&mut self, ppu_event: PpuEvent) -> Vec<FutureEvent> {
        self.io_registers.ppu_mut().handle_event(ppu_event)
    }
}
