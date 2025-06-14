use std::{cell::RefCell, rc::Rc};

use ironboyadvance_arm7tdmi::memory::{
    IoMemoryAccess, MemoryAccess, MemoryAccessWidth, MemoryInterface, decompose_access_pattern,
};

use crate::{
    bios::Bios,
    cartridge::Cartridge,
    io_registers::{IoRegisters, WaitStateControl},
    scheduler::Scheduler,
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

// Indices for cycles lut
pub const INDEX_WRAM_BOARD: usize = (WRAM_BOARD_BASE >> 24) as usize;
pub const INDEX_PALETTE_RAM: usize = (PALETTE_RAM_BASE >> 24) as usize;
pub const INDEX_VRAM: usize = (VRAM_BASE >> 24) as usize;
pub const INDEX_ROM_WS0: usize = (ROM_WS0_LO >> 24) as usize;
pub const INDEX_ROM_WS1: usize = (ROM_WS1_LO >> 24) as usize;
pub const INDEX_ROM_WS2: usize = (ROM_WS2_LO >> 24) as usize;
pub const INDEX_SRAM_LO: usize = (SRAM_LO >> 24) as usize;

pub const GAMEPAK_NON_SEQUENTIAL_CYCLES: [usize; 4] = [4, 3, 2, 8];
pub const GAMEPAK_WS0_SEQUENTIAL_CYCLES: [usize; 2] = [2, 1];
pub const GAMEPAK_WS1_SEQUENTIAL_CYCLES: [usize; 2] = [4, 1];
pub const GAMEPAK_WS2_SEQUENTIAL_CYCLES: [usize; 2] = [4, 1];

pub struct ClockCycleLuts {
    n_cycles_32_lut: [usize; 16],
    s_cycles_32_lut: [usize; 16],
    n_cycles_16_lut: [usize; 16],
    s_cycles_16_lut: [usize; 16],
}

impl ClockCycleLuts {
    pub fn new() -> Self {
        let mut n_cycles_16_lut = [1; 16];
        let mut s_cycles_16_lut = [1; 16];
        let mut n_cycles_32_lut = [1; 16];
        let mut s_cycles_32_lut = [1; 16];

        n_cycles_32_lut[INDEX_WRAM_BOARD] = 6;
        s_cycles_32_lut[INDEX_WRAM_BOARD] = 6;
        n_cycles_16_lut[INDEX_WRAM_BOARD] = 3;
        s_cycles_16_lut[INDEX_WRAM_BOARD] = 3;

        n_cycles_32_lut[INDEX_PALETTE_RAM] = 2;
        s_cycles_32_lut[INDEX_PALETTE_RAM] = 2;

        n_cycles_32_lut[INDEX_VRAM] = 2;
        s_cycles_32_lut[INDEX_VRAM] = 2;

        for i in 0..2 {
            n_cycles_16_lut[INDEX_ROM_WS0 + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            n_cycles_16_lut[INDEX_ROM_WS1 + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            n_cycles_16_lut[INDEX_ROM_WS2 + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];

            s_cycles_16_lut[INDEX_ROM_WS0 + i] = 1 + GAMEPAK_WS0_SEQUENTIAL_CYCLES[0];
            s_cycles_16_lut[INDEX_ROM_WS1 + i] = 1 + GAMEPAK_WS1_SEQUENTIAL_CYCLES[0];
            s_cycles_16_lut[INDEX_ROM_WS2 + i] = 1 + GAMEPAK_WS2_SEQUENTIAL_CYCLES[0];

            // 1N + 1S
            n_cycles_32_lut[INDEX_ROM_WS0 + i] = n_cycles_16_lut[INDEX_ROM_WS0 + i] + s_cycles_16_lut[INDEX_ROM_WS0 + i];
            n_cycles_32_lut[INDEX_ROM_WS1 + i] = n_cycles_16_lut[INDEX_ROM_WS1 + i] + s_cycles_16_lut[INDEX_ROM_WS1 + i];
            n_cycles_32_lut[INDEX_ROM_WS2 + i] = n_cycles_16_lut[INDEX_ROM_WS2 + i] + s_cycles_16_lut[INDEX_ROM_WS2 + i];

            // 2S
            s_cycles_32_lut[INDEX_ROM_WS0 + i] = 2 * s_cycles_16_lut[INDEX_ROM_WS0 + i];
            s_cycles_32_lut[INDEX_ROM_WS1 + i] = 2 * s_cycles_16_lut[INDEX_ROM_WS1 + i];
            s_cycles_32_lut[INDEX_ROM_WS2 + i] = 2 * s_cycles_16_lut[INDEX_ROM_WS2 + i];

            // SRAM
            n_cycles_16_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            n_cycles_32_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            s_cycles_16_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            s_cycles_32_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
        }

        ClockCycleLuts {
            n_cycles_32_lut,
            s_cycles_32_lut,
            n_cycles_16_lut,
            s_cycles_16_lut,
        }
    }

    pub fn update_wait_states(&mut self, waitcnt: &WaitStateControl) {
        let ws0_first_access = waitcnt.ws0_first_access() as usize;
        let ws1_first_access = waitcnt.ws1_first_access() as usize;
        let ws2_first_access = waitcnt.ws2_first_access() as usize;
        let ws0_second_access = waitcnt.ws0_second_access() as usize;
        let ws1_second_access = waitcnt.ws1_second_access() as usize;
        let ws2_second_access = waitcnt.ws2_second_access() as usize;

        for i in 0..2 {
            self.n_cycles_16_lut[INDEX_ROM_WS0 + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[ws0_first_access];
            self.n_cycles_16_lut[INDEX_ROM_WS1 + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[ws1_first_access];
            self.n_cycles_16_lut[INDEX_ROM_WS2 + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[ws2_first_access];

            self.s_cycles_16_lut[INDEX_ROM_WS0 + i] = 1 + GAMEPAK_WS0_SEQUENTIAL_CYCLES[ws0_second_access];
            self.s_cycles_16_lut[INDEX_ROM_WS1 + i] = 1 + GAMEPAK_WS1_SEQUENTIAL_CYCLES[ws1_second_access];
            self.s_cycles_16_lut[INDEX_ROM_WS2 + i] = 1 + GAMEPAK_WS2_SEQUENTIAL_CYCLES[ws2_second_access];

            // 1N + 1S
            self.n_cycles_32_lut[INDEX_ROM_WS0 + i] =
                self.n_cycles_16_lut[INDEX_ROM_WS0 + i] + self.s_cycles_16_lut[INDEX_ROM_WS0 + i];
            self.n_cycles_32_lut[INDEX_ROM_WS1 + i] =
                self.n_cycles_16_lut[INDEX_ROM_WS1 + i] + self.s_cycles_16_lut[INDEX_ROM_WS1 + i];
            self.n_cycles_32_lut[INDEX_ROM_WS2 + i] =
                self.n_cycles_16_lut[INDEX_ROM_WS2 + i] + self.s_cycles_16_lut[INDEX_ROM_WS2 + i];

            // 2S
            self.s_cycles_32_lut[INDEX_ROM_WS0 + i] = 2 * self.s_cycles_16_lut[INDEX_ROM_WS0 + i];
            self.s_cycles_32_lut[INDEX_ROM_WS1 + i] = 2 * self.s_cycles_16_lut[INDEX_ROM_WS1 + i];
            self.s_cycles_32_lut[INDEX_ROM_WS2 + i] = 2 * self.s_cycles_16_lut[INDEX_ROM_WS2 + i];

            // SRAM
            self.n_cycles_16_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            self.n_cycles_32_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            self.s_cycles_16_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
            self.s_cycles_32_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[0];
        }
    }
}

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
    cycle_luts: Rc<RefCell<ClockCycleLuts>>,
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
            IO_REGISTERS_BASE => self.io_registers.read_8(address), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.pallete_ram[(address - PALETTE_RAM_BASE) as usize],
            VRAM_BASE => self.vram[(address - VRAM_BASE) as usize],
            OAM_BASE => self.oam[(address - OAM_BASE) as usize],
            //TODO: move into cart read
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.read_8(address - ROM_WS0_LO),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.read_8(address - ROM_WS1_LO),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.read_8(address - ROM_WS2_LO),
            SRAM_LO | SRAM_HI => self.cartridge.read_8(address - SRAM_LO),
            _ => panic!("Unused: {:08X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address & 0xFF000000 {
            BIOS_BASE => self.bios.write_8(address - BIOS_BASE, value),
            WRAM_BOARD_BASE => self.wram_board[(address - WRAM_BOARD_BASE) as usize] = value,
            WRAM_CHIP_BASE => self.wram_chip[(address - WRAM_CHIP_BASE) as usize] = value,
            IO_REGISTERS_BASE => self.io_registers.write_8(address, value), // theres mirrors for this see GBATEK
            PALETTE_RAM_BASE => self.pallete_ram[(address - PALETTE_RAM_BASE) as usize] = value,
            VRAM_BASE => self.vram[(address - VRAM_BASE) as usize] = value,
            OAM_BASE => self.oam[(address - OAM_BASE) as usize] = value,
            //TODO: move into cart read
            ROM_WS0_LO | ROM_WS0_HI => self.cartridge.write_8(address - ROM_WS0_LO, value),
            ROM_WS1_LO | ROM_WS1_HI => self.cartridge.write_8(address - ROM_WS1_LO, value),
            ROM_WS2_LO | ROM_WS2_HI => self.cartridge.write_8(address - ROM_WS2_LO, value),
            SRAM_LO | SRAM_HI => self.cartridge.write_8(address - SRAM_LO, value),
            _ => panic!("Unused: {:08X}", address),
        }
    }
}

impl SystemBus {
    pub fn new(cartridge: Cartridge, bios: Bios, scheduler: Rc<RefCell<Scheduler>>) -> Self {
        let cycle_luts = Rc::new(RefCell::new(ClockCycleLuts::new()));
        SystemBus {
            bios,
            wram_board: vec![0; 0x40000],
            wram_chip: vec![0; 0x8000],
            io_registers: IoRegisters::new(scheduler.clone(), cycle_luts.clone()), // pass scheduler
            pallete_ram: vec![0; 0x400],
            vram: vec![0; 0x18000],
            oam: vec![0; 0x400],
            cartridge,
            scheduler,
            cycle_luts: cycle_luts,
        }
    }

    pub fn cycle(&mut self, address: u32, access_pattern: u8, width: MemoryAccessWidth) {
        let access = decompose_access_pattern(access_pattern)[0];
        let index = ((address >> 24) & 0xF) as usize;
        let cycles = match width {
            MemoryAccessWidth::Byte | MemoryAccessWidth::HalfWord => match access {
                MemoryAccess::NonSequential => self.cycle_luts.borrow().n_cycles_16_lut[index],
                MemoryAccess::Sequential => self.cycle_luts.borrow().s_cycles_16_lut[index],
                _ => panic!("Should be NonSequential or Sequential"),
            },
            MemoryAccessWidth::Word => match access {
                MemoryAccess::NonSequential => self.cycle_luts.borrow().n_cycles_32_lut[index],
                MemoryAccess::Sequential => self.cycle_luts.borrow().s_cycles_32_lut[index],
                _ => panic!("Should be NonSequential or Sequential"),
            },
        };

        self.scheduler.borrow_mut().update(cycles);
    }
}
