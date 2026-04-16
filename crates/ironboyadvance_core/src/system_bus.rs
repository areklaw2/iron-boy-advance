use std::{cell::RefCell, rc::Rc};

use getset::{Getters, MutGetters};
use ironboyadvance_arm7tdmi::{
    CpuState,
    memory::{MemoryAccessWidth, MemoryInterface, SystemMemoryAccess, decompose_access_pattern},
};
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
    cpu_state: CpuState,
    pc: u32,
    pipeline: [u32; 2],
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

    fn update_pc_ref(&mut self, pc: u32) {
        self.bios.set_pc_ref(pc);
        self.pc = pc;
    }

    fn update_cpu_state_ref(&mut self, cpu_state: CpuState) {
        self.cpu_state = cpu_state
    }

    fn update_pipeline_ref(&mut self, decoded: u32, prefetched: u32) {
        self.pipeline = [decoded, prefetched];
    }
}

impl SystemMemoryAccess for SystemBus {
    fn read_8(&self, address: u32) -> u8 {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_8(address),
            WRAM_BOARD_BASE => self.memory.read_8(address),
            WRAM_CHIP_BASE => self.memory.read_8(address),
            IO_REGISTERS_BASE => self.io_registers.read_8(address),
            PALETTE_RAM_BASE => self.io_registers.read_8(address),
            VRAM_BASE => self.io_registers.read_8(address),
            OAM_BASE => self.io_registers.read_8(address),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_8(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_8(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_8(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_8(address),
            _ => {
                debug!("Unused Read from {:08X}", address);
                (self.open_bus_read_32() >> ((address & 3) * 8)) as u8
            }
        }
    }

    fn read_16(&self, address: u32) -> u16 {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_16(address),
            WRAM_BOARD_BASE => self.memory.read_16(address),
            WRAM_CHIP_BASE => self.memory.read_16(address),
            IO_REGISTERS_BASE => self.io_registers.read_16(address),
            PALETTE_RAM_BASE => self.io_registers.read_16(address),
            VRAM_BASE => self.io_registers.read_16(address),
            OAM_BASE => self.io_registers.read_16(address),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_16(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_16(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_16(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_16(address),
            _ => {
                debug!("Unused Read from {:08X}", address);
                (self.open_bus_read_32() >> ((address & 2) * 8)) as u16
            }
        }
    }

    fn read_32(&self, address: u32) -> u32 {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_32(address),
            WRAM_BOARD_BASE => self.memory.read_32(address),
            WRAM_CHIP_BASE => self.memory.read_32(address),
            IO_REGISTERS_BASE => self.io_registers.read_32(address),
            PALETTE_RAM_BASE => self.io_registers.read_32(address),
            VRAM_BASE => self.io_registers.read_32(address),
            OAM_BASE => self.io_registers.read_32(address),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_32(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_32(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_32(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_32(address),
            _ => {
                debug!("Unused Read from {:08X}", address);
                self.open_bus_read_32()
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_8(address, value),
            WRAM_BOARD_BASE => self.memory.write_8(address, value),
            WRAM_CHIP_BASE => self.memory.write_8(address, value),
            IO_REGISTERS_BASE => self.io_registers.write_8(address, value),
            PALETTE_RAM_BASE => self.io_registers.write_8(address, value),
            VRAM_BASE => self.io_registers.write_8(address, value),
            OAM_BASE => self.io_registers.write_8(address, value),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_8(address, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_8(address, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_8(address, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_8(address, value),
            _ => debug!("Unused Write {} to {:08X}", value, address),
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_16(address, value),
            WRAM_BOARD_BASE => self.memory.write_16(address, value),
            WRAM_CHIP_BASE => self.memory.write_16(address, value),
            IO_REGISTERS_BASE => self.io_registers.write_16(address, value),
            PALETTE_RAM_BASE => self.io_registers.write_16(address, value),
            VRAM_BASE => self.io_registers.write_16(address, value),
            OAM_BASE => self.io_registers.write_16(address, value),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_16(address, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_16(address, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_16(address, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_16(address, value),
            _ => debug!("Unused Write {} to {:08X}", value, address),
        }
    }

    fn write_32(&mut self, address: u32, value: u32) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_32(address, value),
            WRAM_BOARD_BASE => self.memory.write_32(address, value),
            WRAM_CHIP_BASE => self.memory.write_32(address, value),
            IO_REGISTERS_BASE => self.io_registers.write_32(address, value),
            PALETTE_RAM_BASE => self.io_registers.write_32(address, value),
            VRAM_BASE => self.io_registers.write_32(address, value),
            OAM_BASE => self.io_registers.write_32(address, value),
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_32(address, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_32(address, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_32(address, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_32(address, value),
            _ => debug!("Unused Write {} to {:08X}", value, address),
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
            cpu_state: CpuState::Arm,
            pc: 0,
            pipeline: [0; 2],
        }
    }

    fn open_bus_read_32(&self) -> u32 {
        //TODO: add dma check
        match self.cpu_state {
            CpuState::Arm => self.pipeline[1],
            CpuState::Thumb => {
                let decoded = self.pipeline[0] & 0xFFFF;
                let fetched = self.pipeline[1] & 0xFFFF;
                let pc = self.pc;
                match pc & 0xFF00_0000 {
                    // Approximation, cant get to $+6 for aligned and $+2 for unaligned
                    // See GBATEK - GBA Unpredictable Things.
                    BIOS_BASE | OAM_BASE => match pc & 2 == 0 {
                        true => (fetched << 16) | decoded,
                        false => (fetched << 16) | fetched,
                    },
                    WRAM_CHIP_BASE => match pc & 2 == 0 {
                        true => (decoded << 16) | fetched,
                        false => (fetched << 16) | decoded,
                    },
                    _ => (fetched << 16) | fetched,
                }
            }
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
