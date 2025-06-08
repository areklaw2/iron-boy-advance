use std::{cell::RefCell, rc::Rc};

use ironboyadvance_arm7tdmi::memory::{IoMemoryAccess, MemoryAccessWidth, MemoryInterface, decompose_access_pattern};

use crate::{bios::Bios, cartridge::Cartridge, io_registers::IoRegisters, scheduler::Scheduler};

pub const BIOS_BASE: u32 = 0x0000_0000;
pub const WRAM_BOARD_BASE: u32 = 0x0200_0000;
pub const WRAM_CHIP_BASE: u32 = 0x0300_0000;
pub const IO_REGISTERS_BASE: u32 = 0x0400_0000;
pub const PALETTE_RAM_BASE: u32 = 0x0500_0000;
pub const VRAM_BASE: u32 = 0x0600_0000;
pub const OAM_BASE: u32 = 0x0700_0000;
pub const ROM_WAIT_STATE_0_LO: u32 = 0x0800_0000;
pub const ROM_WAIT_STATE_0_HI: u32 = 0x0900_0000;
pub const ROM_WAIT_STATE_1_LO: u32 = 0x0A00_0000;
pub const ROM_WAIT_STATE_1_HI: u32 = 0x0B00_0000;
pub const ROM_WAIT_STATE_2_LO: u32 = 0x0C00_0000;
pub const ROM_WAIT_STATE_2_HI: u32 = 0x0D00_0000;
pub const SRAM_LO: u32 = 0x0E00_0000;
pub const SRAM_HI: u32 = 0x0F00_0000;

// Indices for cycles lut
pub const INDEX_BIOS: usize = (BIOS_BASE >> 24) as usize;
pub const INDEX_WRAM_BOARD: usize = (WRAM_BOARD_BASE >> 24) as usize;
pub const INDEX_WRAM_CHIP: usize = (WRAM_CHIP_BASE >> 24) as usize;
pub const INDEX_IO_REGISTERS: usize = (IO_REGISTERS_BASE >> 24) as usize;
pub const INDEX_PALETTE_RAM: usize = (PALETTE_RAM_BASE >> 24) as usize;
pub const INDEX_VRAM: usize = (VRAM_BASE >> 24) as usize;
pub const INDEX_OAM: usize = (OAM_BASE >> 24) as usize;
pub const INDEX_ROM_WAIT_STATE_0: usize = (ROM_WAIT_STATE_0_LO >> 24) as usize;
pub const INDEX_ROM_WAIT_STATE_1: usize = (ROM_WAIT_STATE_1_LO >> 24) as usize;
pub const INDEX_ROM_WAIT_STATE_2: usize = (ROM_WAIT_STATE_2_LO >> 24) as usize;
pub const INDEX_SRAM_LO: usize = (SRAM_LO >> 24) as usize;
pub const INDEX_SRAM_HI: usize = (SRAM_HI >> 24) as usize;

pub struct SystemBus {
    bios: Bios,
    wram_board: Vec<u8>,
    wram_chip: Vec<u8>,
    io_registers: IoRegisters,
    // TODO: Probably need to add this to ppu
    pallete_ram: Vec<u8>,
    vram: Vec<u8>,
    oam: Vec<u8>,
    cartridge: Cartridge,
    scheduler: Rc<RefCell<Scheduler>>,
    // TODO: Add luts for cycles
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
        self.scheduler.borrow_mut().update(1);
    }
}

impl IoMemoryAccess for SystemBus {
    fn read_8(&self, address: u32) -> u8 {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_8(address - BIOS_BASE),
            WRAM_BOARD_BASE => self.wram_board[(address - WRAM_BOARD_BASE) as usize],
            WRAM_CHIP_BASE => self.wram_chip[(address - WRAM_CHIP_BASE) as usize],
            IO_REGISTERS_BASE => self.io_registers.read_8(address - IO_REGISTERS_BASE), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.pallete_ram[(address - PALETTE_RAM_BASE) as usize],
            VRAM_BASE => self.vram[(address - VRAM_BASE) as usize],
            OAM_BASE => self.oam[(address - OAM_BASE) as usize],
            //TODO: move into cart read
            ROM_WAIT_STATE_0_LO | ROM_WAIT_STATE_0_HI => self.cartridge.read_8(address - ROM_WAIT_STATE_0_LO),
            ROM_WAIT_STATE_1_LO | ROM_WAIT_STATE_1_HI => self.cartridge.read_8(address - ROM_WAIT_STATE_1_LO),
            ROM_WAIT_STATE_2_LO | ROM_WAIT_STATE_2_HI => self.cartridge.read_8(address - ROM_WAIT_STATE_2_LO),
            SRAM_LO | SRAM_HI => self.cartridge.read_8(address - SRAM_LO),
            _ => panic!("Unused: {:08X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_8(address - BIOS_BASE, value),
            WRAM_BOARD_BASE => self.wram_board[(address - WRAM_BOARD_BASE) as usize] = value,
            WRAM_CHIP_BASE => self.wram_chip[(address - WRAM_CHIP_BASE) as usize] = value,
            IO_REGISTERS_BASE => self.io_registers.write_8(address - IO_REGISTERS_BASE, value), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.pallete_ram[(address - PALETTE_RAM_BASE) as usize] = value,
            VRAM_BASE => self.vram[(address - VRAM_BASE) as usize] = value,
            OAM_BASE => self.oam[(address - OAM_BASE) as usize] = value,
            //TODO: move into cart read
            ROM_WAIT_STATE_0_LO | ROM_WAIT_STATE_0_HI => self.cartridge.write_8(address - ROM_WAIT_STATE_0_LO, value),
            ROM_WAIT_STATE_1_LO | ROM_WAIT_STATE_1_HI => self.cartridge.write_8(address - ROM_WAIT_STATE_1_LO, value),
            ROM_WAIT_STATE_2_LO | ROM_WAIT_STATE_2_HI => self.cartridge.write_8(address - ROM_WAIT_STATE_2_LO, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_8(address - SRAM_LO, value),
            _ => panic!("Unused: {:08X}", address),
        }
    }
}

impl SystemBus {
    pub fn new(cartridge: Cartridge, bios: Bios, scheduler: Rc<RefCell<Scheduler>>) -> Self {
        SystemBus {
            bios,
            wram_board: vec![0; 0x40000],
            wram_chip: vec![0; 0x8000],
            io_registers: IoRegisters::new(),
            pallete_ram: vec![0; 0x400],
            vram: vec![0; 0x18000],
            oam: vec![0; 0x400],
            cartridge,
            scheduler,
        }
    }

    pub fn cycle(&mut self, address: u32, access_pattern: u8, width: MemoryAccessWidth) {
        let access = decompose_access_pattern(access_pattern);
        println!("Do stuff with 0x{:08X}, {:?}, {:?}", address, access, width);
        self.scheduler.borrow_mut().update(1);
    }
}
