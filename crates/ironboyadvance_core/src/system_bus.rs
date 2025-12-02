use std::{cell::RefCell, rc::Rc};

use ironboyadvance_arm7tdmi::memory::{MemoryAccessWidth, MemoryInterface, SystemMemoryAccess, decompose_access_pattern};

use crate::{bios::Bios, cartridge::Cartridge, io_registers::IoRegisters, scheduler::Scheduler, system_control::HaltMode};

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

pub struct SystemBus {
    bios: Bios,
    wram_board: Vec<u8>,
    wram_chip: Vec<u8>,
    io_registers: IoRegisters, //TODO: make getter
    // TODO: Probably need to add this to ppu
    pallete_ram: Vec<u8>,
    vram: Vec<u8>,
    oam: Vec<u8>,
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
        self.scheduler.borrow_mut().update(1);
    }
}

impl SystemMemoryAccess for SystemBus {
    fn read_8(&self, address: u32) -> u8 {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.read_8(address),
            WRAM_BOARD_BASE => self.wram_board[(address & 0x3FFFF) as usize],
            WRAM_CHIP_BASE => self.wram_chip[(address & 0x7FFF) as usize],
            IO_REGISTERS_BASE => self.io_registers.read_8(address), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.pallete_ram[(address & 0x3FF) as usize],
            VRAM_BASE => self.vram[(address & 0x17FFF) as usize],
            OAM_BASE => self.oam[(address & 0x3FF) as usize],
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_8(address),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_8(address),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_8(address),
            SRAM_LO | SRAM_HI => self.cartridge.read_8(address),
            _ => panic!("Unused: {:08X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_8(address, value),
            WRAM_BOARD_BASE => self.wram_board[(address & 0x3FFFF) as usize] = value,
            WRAM_CHIP_BASE => self.wram_chip[(address & 0x7FFF) as usize] = value,
            IO_REGISTERS_BASE => self.io_registers.write_8(address, value), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.pallete_ram[(address & 0x3FF) as usize] = value,
            VRAM_BASE => self.vram[(address & 0x17FFF) as usize] = value,
            OAM_BASE => self.oam[(address & 0x3FF) as usize] = value,
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
        SystemBus {
            bios,
            wram_board: vec![0; 0x40000],
            wram_chip: vec![0; 0x8000],
            io_registers: IoRegisters::new(scheduler.clone()),
            pallete_ram: vec![0; 0x400],
            vram: vec![0; 0x18000],
            oam: vec![0; 0x400],
            cartridge,
            scheduler,
        }
    }

    pub fn cycle(&mut self, address: u32, access_pattern: u8, width: MemoryAccessWidth) {
        let access = decompose_access_pattern(access_pattern)[0];
        let index = ((address >> 24) & 0xF) as usize;
        let cycles = self.io_registers.system_controller().cycles(index, width, access);
        self.scheduler.borrow_mut().update(cycles);
    }

    pub fn interrupt_pending(&self) -> bool {
        self.io_registers.interrupt_pending()
    }

    pub fn halt_mode(&self) -> HaltMode {
        self.io_registers.system_controller().halt_mode()
    }

    pub fn un_halt(&mut self) {
        //TODO: this needs to check interrupts as well
        unimplemented!();
    }
}

#[inline]
pub fn read_reg_16_byte(value: u16, address: u32) -> u8 {
    match address & 1 {
        0 => (value & 0xFF) as u8,
        _ => ((value >> 8) & 0xFF) as u8,
    }
}

#[inline]
pub fn write_reg_16_byte(current: u16, address: u32, value: u8) -> u16 {
    match address & 1 {
        0 => (current & 0xFF00) | (value as u16),
        _ => (current & 0x00FF) | ((value as u16) << 8),
    }
}

#[inline]
pub fn read_reg_32_byte(value: u32, address: u32) -> u8 {
    let byte_offset = (address & 3) * 8;
    ((value >> byte_offset) & 0xFF) as u8
}

#[inline]
pub fn write_u32_byte(current: u32, address: u32, value: u8) -> u32 {
    let byte_offset = (address & 3) * 8;
    let mask = !(0xFF << byte_offset);
    (current & mask) | ((value as u32) << byte_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_reg_16_byte_low() {
        let value: u16 = 0x1234;
        assert_eq!(read_reg_16_byte(value, 0x04000200), 0x34);
        assert_eq!(read_reg_16_byte(value, 0x00000000), 0x34);
    }

    #[test]
    fn test_read_reg_16_byte_high() {
        let value: u16 = 0x1234;
        assert_eq!(read_reg_16_byte(value, 0x04000201), 0x12);
        assert_eq!(read_reg_16_byte(value, 0x00000001), 0x12);
    }

    #[test]
    fn test_write_reg_16_byte_low() {
        let current: u16 = 0x1234;
        let result = write_reg_16_byte(current, 0x04000200, 0xAB);
        assert_eq!(result, 0x12AB);
    }

    #[test]
    fn test_write_reg_16_byte_high() {
        let current: u16 = 0x1234;
        let result = write_reg_16_byte(current, 0x04000201, 0xAB);
        assert_eq!(result, 0xAB34);
    }

    #[test]
    fn test_write_reg_16_byte_preserves_other_byte() {
        let current: u16 = 0xFFFF;
        let result = write_reg_16_byte(current, 0x00000000, 0x00);
        assert_eq!(result, 0xFF00);

        let result = write_reg_16_byte(current, 0x00000001, 0x00);
        assert_eq!(result, 0x00FF);
    }

    #[test]
    fn test_read_reg_32_byte() {
        let value: u32 = 0x12345678;

        assert_eq!(read_reg_32_byte(value, 0x04000800), 0x78);
        assert_eq!(read_reg_32_byte(value, 0x04000801), 0x56);
        assert_eq!(read_reg_32_byte(value, 0x04000802), 0x34);
        assert_eq!(read_reg_32_byte(value, 0x04000803), 0x12);
    }

    #[test]
    fn test_write_u32_byte() {
        let current: u32 = 0x12345678;

        let result = write_u32_byte(current, 0x04000800, 0xAA);
        assert_eq!(result, 0x123456AA);

        let result = write_u32_byte(current, 0x04000801, 0xBB);
        assert_eq!(result, 0x1234BB78);

        let result = write_u32_byte(current, 0x04000802, 0xCC);
        assert_eq!(result, 0x12CC5678);

        let result = write_u32_byte(current, 0x04000803, 0xDD);
        assert_eq!(result, 0xDD345678);
    }

    #[test]
    fn test_write_u32_byte_preserves_other_bytes() {
        let current: u32 = 0xFFFFFFFF;

        let result = write_u32_byte(current, 0x00000000, 0x00);
        assert_eq!(result, 0xFFFFFF00);

        let result = write_u32_byte(current, 0x00000001, 0x00);
        assert_eq!(result, 0xFFFF00FF);

        let result = write_u32_byte(current, 0x00000002, 0x00);
        assert_eq!(result, 0xFF00FFFF);

        let result = write_u32_byte(current, 0x00000003, 0x00);
        assert_eq!(result, 0x00FFFFFF);
    }

    #[test]
    fn test_write_read_16_byte() {
        let original: u16 = 0xABCD;

        let after_write = write_reg_16_byte(0x0000, 0x00000000, 0xCD);
        assert_eq!(read_reg_16_byte(after_write, 0x00000000), 0xCD);

        let after_write = write_reg_16_byte(after_write, 0x00000001, 0xAB);
        assert_eq!(read_reg_16_byte(after_write, 0x00000001), 0xAB);
        assert_eq!(after_write, original);
    }

    #[test]
    fn test_write_read_32_byte() {
        let original: u32 = 0x12345678;
        let mut value: u32 = 0x00000000;

        // Write all bytes
        value = write_u32_byte(value, 0x00000000, 0x78);
        value = write_u32_byte(value, 0x00000001, 0x56);
        value = write_u32_byte(value, 0x00000002, 0x34);
        value = write_u32_byte(value, 0x00000003, 0x12);

        // Read them back
        assert_eq!(read_reg_32_byte(value, 0x00000000), 0x78);
        assert_eq!(read_reg_32_byte(value, 0x00000001), 0x56);
        assert_eq!(read_reg_32_byte(value, 0x00000002), 0x34);
        assert_eq!(read_reg_32_byte(value, 0x00000003), 0x12);
        assert_eq!(value, original);
    }
}
