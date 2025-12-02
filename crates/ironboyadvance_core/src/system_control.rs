use bitfields::bitfield;
use getset::{CopyGetters, Setters};
use ironboyadvance_arm7tdmi::memory::{MemoryAccess, MemoryAccessWidth, SystemMemoryAccess};
use tracing::debug;

use crate::system_bus::{
    PALETTE_RAM_BASE, ROM_WS0_LO, ROM_WS1_LO, ROM_WS2_LO, SRAM_LO, VRAM_BASE, WRAM_BOARD_BASE, read_reg_16_byte,
    read_reg_32_byte, write_reg_16_byte, write_u32_byte,
};

const INDEX_WRAM_BOARD: usize = (WRAM_BOARD_BASE >> 24) as usize;
const INDEX_PALETTE_RAM: usize = (PALETTE_RAM_BASE >> 24) as usize;
const INDEX_VRAM: usize = (VRAM_BASE >> 24) as usize;
const INDEX_ROM_WS0: usize = (ROM_WS0_LO >> 24) as usize;
const INDEX_ROM_WS1: usize = (ROM_WS1_LO >> 24) as usize;
const INDEX_ROM_WS2: usize = (ROM_WS2_LO >> 24) as usize;
const INDEX_SRAM_LO: usize = (SRAM_LO >> 24) as usize;

const GAMEPAK_NON_SEQUENTIAL_CYCLES: [usize; 4] = [4, 3, 2, 8];
const GAMEPAK_WS0_SEQUENTIAL_CYCLES: [usize; 2] = [2, 1];
const GAMEPAK_WS1_SEQUENTIAL_CYCLES: [usize; 2] = [4, 1];
const GAMEPAK_WS2_SEQUENTIAL_CYCLES: [usize; 2] = [8, 1];

struct ClockCycleLuts {
    n_cycles_32_lut: [usize; 16],
    s_cycles_32_lut: [usize; 16],
    n_cycles_16_lut: [usize; 16],
    s_cycles_16_lut: [usize; 16],
}

impl ClockCycleLuts {
    fn new() -> Self {
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

    fn update_wait_states(&mut self, waitcnt: &WaitStateControl) {
        let ws0_first_access = waitcnt.ws0_first_access() as usize;
        let ws1_first_access = waitcnt.ws1_first_access() as usize;
        let ws2_first_access = waitcnt.ws2_first_access() as usize;
        let ws0_second_access = waitcnt.ws0_second_access() as usize;
        let ws1_second_access = waitcnt.ws1_second_access() as usize;
        let ws2_second_access = waitcnt.ws2_second_access() as usize;
        let sram_wait_control = waitcnt.sram_wait_control() as usize;

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
            self.n_cycles_16_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[sram_wait_control];
            self.n_cycles_32_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[sram_wait_control];
            self.s_cycles_16_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[sram_wait_control];
            self.s_cycles_32_lut[INDEX_SRAM_LO + i] = 1 + GAMEPAK_NON_SEQUENTIAL_CYCLES[sram_wait_control];
        }
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
struct WaitStateControl {
    #[bits(2)]
    sram_wait_control: u8,
    #[bits(2)]
    ws0_first_access: u8,
    ws0_second_access: bool,
    #[bits(2)]
    ws1_first_access: u8,
    ws1_second_access: bool,
    #[bits(2)]
    ws2_first_access: u8,
    ws2_second_access: bool,
    #[bits(2)]
    phi_terminal_output: u8,
    _reserved: bool,
    game_pak_prefetch_buffer_enable: bool,
    game_pak_type_flag: bool,
}

#[bitfield(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
struct InternalMemoryControl {
    disable_32k_256k_wram: bool,
    #[bits(3)]
    unknown_1: u8,
    _reserved_1: bool,
    enable_256_wram: bool,
    #[bits(18)]
    _reserved_2: u32,
    #[bits(4)]
    wait_control_wram: u8,
    #[bits(4)]
    unknown_2: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum HaltMode {
    Halted,
    Stopped,
}

#[derive(CopyGetters, Setters)]
pub struct SystemController {
    cycle_luts: ClockCycleLuts,
    waitstate_control: WaitStateControl,
    post_flag: bool,
    #[getset(get_copy = "pub", set = "pub")]
    halt_mode: HaltMode,
    internal_memory_control: InternalMemoryControl,
}

impl SystemController {
    pub fn new() -> Self {
        SystemController {
            cycle_luts: ClockCycleLuts::new(),
            waitstate_control: WaitStateControl::from_bits(0),
            post_flag: false,
            halt_mode: HaltMode::Halted,
            internal_memory_control: InternalMemoryControl::from_bits(0x0D000020), // Initialized by hardware
        }
    }

    pub fn set_waitstate_control(&mut self, value: u16) {
        self.waitstate_control.set_bits(value);
        self.cycle_luts.update_wait_states(&self.waitstate_control);
    }

    pub fn cycles(&self, lut_index: usize, width: MemoryAccessWidth, access: MemoryAccess) -> usize {
        match width {
            MemoryAccessWidth::Byte | MemoryAccessWidth::HalfWord => match access {
                MemoryAccess::NonSequential => self.cycle_luts.n_cycles_16_lut[lut_index],
                MemoryAccess::Sequential => self.cycle_luts.s_cycles_16_lut[lut_index],
                _ => panic!("Should be NonSequential or Sequential"),
            },
            MemoryAccessWidth::Word => match access {
                MemoryAccess::NonSequential => self.cycle_luts.n_cycles_32_lut[lut_index],
                MemoryAccess::Sequential => self.cycle_luts.s_cycles_32_lut[lut_index],
                _ => panic!("Should be NonSequential or Sequential"),
            },
        }
    }
}

impl SystemMemoryAccess for SystemController {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // WAITCNT
            0x04000204..=0x04000205 => read_reg_16_byte(self.waitstate_control.into_bits(), address),
            // POSTFLG
            0x04000300 => self.post_flag as u8,
            // INTERNAL MEMORY CONTROL
            0x04000800..=0x04000803 => read_reg_32_byte(self.internal_memory_control.into_bits(), address),
            _ => panic!("Invalid byte read for SystemController: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // WAITCNT
            0x04000204..=0x04000205 => {
                let new_value = write_reg_16_byte(self.waitstate_control.into_bits(), address, value);
                self.set_waitstate_control(new_value);
            }
            // POSTFLG
            0x04000300 => self.post_flag = value & 0x1 != 0,
            // HALTCNT
            0x04000301 => match value & 0x80 != 0 {
                true => todo!("Stopped"),
                false => self.halt_mode = HaltMode::Halted,
            },
            // INTERNAL MEMORY CONTROL
            0x04000800..=0x04000803 => {
                let new_value = write_u32_byte(self.internal_memory_control.into_bits(), address, value);
                self.internal_memory_control.set_bits(new_value);
            }
            _ => debug!("Invalid byte write in SystemController: {:#010X}", address),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{system_control::ClockCycleLuts, system_control::WaitStateControl};

    #[test]
    fn clock_cycles() {
        let mut clock_cycle_luts = ClockCycleLuts::new();
        assert_eq!(
            clock_cycle_luts.n_cycles_16_lut,
            [1, 1, 3, 1, 1, 1, 1, 1, 5, 5, 5, 5, 5, 5, 5, 5]
        );
        assert_eq!(
            clock_cycle_luts.s_cycles_16_lut,
            [1, 1, 3, 1, 1, 1, 1, 1, 3, 3, 5, 5, 9, 9, 5, 5]
        );
        assert_eq!(
            clock_cycle_luts.n_cycles_32_lut,
            [1, 1, 6, 1, 1, 2, 2, 1, 8, 8, 10, 10, 14, 14, 5, 5]
        );
        assert_eq!(
            clock_cycle_luts.s_cycles_32_lut,
            [1, 1, 6, 1, 1, 2, 2, 1, 6, 6, 10, 10, 18, 18, 5, 5]
        );

        clock_cycle_luts.update_wait_states(&WaitStateControl::from_bits(0b100001100010111));
        assert_eq!(
            clock_cycle_luts.n_cycles_16_lut,
            [1, 1, 3, 1, 1, 1, 1, 1, 4, 4, 5, 5, 9, 9, 9, 9]
        );
        assert_eq!(
            clock_cycle_luts.s_cycles_16_lut,
            [1, 1, 3, 1, 1, 1, 1, 1, 2, 2, 5, 5, 9, 9, 9, 9]
        );
        assert_eq!(
            clock_cycle_luts.n_cycles_32_lut,
            [1, 1, 6, 1, 1, 2, 2, 1, 6, 6, 10, 10, 18, 18, 9, 9]
        );
        assert_eq!(
            clock_cycle_luts.s_cycles_32_lut,
            [1, 1, 6, 1, 1, 2, 2, 1, 4, 4, 10, 10, 18, 18, 9, 9]
        );
    }
}
