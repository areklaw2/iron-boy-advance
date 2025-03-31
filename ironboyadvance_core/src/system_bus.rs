use std::{cell::RefCell, rc::Rc};

use ironboyadvance_arm7tdmi::memory::{IoMemoryAccess, MemoryAccessWidth, MemoryInterface, decompose_access_pattern};

use crate::{bios::Bios, cartridge::Cartridge, scheduler::Scheduler};

pub const BIOS_START: u32 = 0x0000_0000;
pub const BIOS_END: u32 = 0x0000_3FFF;
pub const WRAM_BOARD_START: u32 = 0x0200_0000;
pub const WRAM_BOARD_END: u32 = 0x0203_FFFF;
pub const WRAM_CHIP_START: u32 = 0x0300_0000;
pub const WRAM_CHIP_END: u32 = 0x0300_7FFF;
pub const IO_REGISTER_START: u32 = 0x0400_0000;
pub const IO_REGISTER_END: u32 = 0x0400_0800;
pub const PALETTE_RAM_START: u32 = 0x0500_0000;
pub const PALETTE_RAM_END: u32 = 0x0500_03FF;
pub const VRAM_START: u32 = 0x0600_0000;
pub const VRAM_END: u32 = 0x0600_7FFF;
pub const OAM_START: u32 = 0x0700_0000;
pub const OAM_END: u32 = 0x0700_03FF;
pub const ROM_WAIT_STATE_0_START: u32 = 0x0800_0000;
pub const ROM_WAIT_STATE_0_END: u32 = 0x09FF_FFFF;
pub const ROM_WAIT_STATE_1_START: u32 = 0x0A00_0000;
pub const ROM_WAIT_STATE_1_END: u32 = 0x0BFF_FFFF;
pub const ROM_WAIT_STATE_2_START: u32 = 0x0C00_0000;
pub const ROM_WAIT_STATE_2_END: u32 = 0x0DFF_FFFF;
pub const SRAM_START: u32 = 0x0E00_0000;
pub const SRAM_END: u32 = 0x0FFF_FFFF;

pub struct SystemBus {
    data: Vec<u8>,
    cartridge: Cartridge,
    bios: Bios,
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
        //update the scheduler
    }
}

impl IoMemoryAccess for SystemBus {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            BIOS_START..=BIOS_END => self.bios.read_8(address),
            WRAM_BOARD_START..=WRAM_BOARD_END => self.data[address as usize],
            WRAM_CHIP_START..=WRAM_CHIP_END => self.data[address as usize],
            IO_REGISTER_START..=IO_REGISTER_END => self.data[address as usize], // theres mirrors for this see GBATEK
            PALETTE_RAM_START..=PALETTE_RAM_END => self.data[address as usize],
            VRAM_START..=VRAM_END => self.data[address as usize],
            OAM_START..=OAM_END => self.data[address as usize],
            ROM_WAIT_STATE_0_START..=ROM_WAIT_STATE_0_END => self.cartridge.read_8(address),
            ROM_WAIT_STATE_1_START..=ROM_WAIT_STATE_1_END => self.cartridge.read_8(address),
            ROM_WAIT_STATE_2_START..=ROM_WAIT_STATE_2_END => self.cartridge.read_8(address),
            SRAM_START..=SRAM_END => self.cartridge.read_8(address),
            _ => self.data[address as usize],
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            BIOS_START..=BIOS_END => self.bios.write_8(address, value),
            WRAM_BOARD_START..=WRAM_BOARD_END => self.data[address as usize] = value,
            WRAM_CHIP_START..=WRAM_CHIP_END => self.data[address as usize] = value,
            IO_REGISTER_START..=IO_REGISTER_END => self.data[address as usize] = value, // theres mirrors for this see GBATEK
            PALETTE_RAM_START..=PALETTE_RAM_END => self.data[address as usize] = value,
            VRAM_START..=VRAM_END => self.data[address as usize] = value,
            OAM_START..=OAM_END => self.data[address as usize] = value,
            ROM_WAIT_STATE_0_START..=ROM_WAIT_STATE_0_END => self.cartridge.write_8(address, value),
            ROM_WAIT_STATE_1_START..=ROM_WAIT_STATE_1_END => self.cartridge.write_8(address, value),
            ROM_WAIT_STATE_2_START..=ROM_WAIT_STATE_2_END => self.cartridge.write_8(address, value),
            SRAM_START..=SRAM_END => self.cartridge.write_8(address, value),
            _ => self.data[address as usize] = value,
        }
    }
}

impl SystemBus {
    pub fn new(cartridge: Cartridge, bios: Bios, scheduler: Rc<RefCell<Scheduler>>) -> Self {
        SystemBus {
            data: vec![0; 0xFFFFFFFF],
            cartridge,
            bios,
            scheduler,
        }
    }

    pub fn cycle(&mut self, address: u32, access_pattern: u8, width: MemoryAccessWidth) {
        let access = decompose_access_pattern(access_pattern);
        println!("Do stuff with 0x{:08X}, {:?}, {:?}", address, access, width);
        self.scheduler.borrow_mut().update(1);
    }
}
