use bit::BitIndex;

use crate::mode::{CpuMode, CpuState};

pub struct ProgramStatusRegister {
    value: u32,
}

impl ProgramStatusRegister {
    pub fn new(value: u32) -> Self {
        ProgramStatusRegister {
            value: value & !0x0FFFFF00,
        }
    }

    pub fn get(&self) -> u32 {
        self.value
    }

    pub fn set(&mut self, value: u32) {
        self.value = value & !0x0FFFFF00
    }

    pub fn set_flags(&mut self, value: u32) {
        self.value &= !0xF0000000;
        self.value |= 0xF0000000 & value;
    }

    pub fn get_n_flag(&self) -> bool {
        self.value.bit(31)
    }

    pub fn set_n_flag(&mut self, status: bool) {
        self.value.set_bit(31, status);
    }

    pub fn get_z_flag(&self) -> bool {
        self.value.bit(30)
    }

    pub fn set_z_flag(&mut self, status: bool) {
        self.value.set_bit(30, status);
    }

    pub fn get_c_flag(&self) -> bool {
        self.value.bit(29)
    }

    pub fn set_c_flag(&mut self, status: bool) {
        self.value.set_bit(29, status);
    }

    pub fn get_v_flag(&self) -> bool {
        self.value.bit(28)
    }

    pub fn set_v_flag(&mut self, status: bool) {
        self.value.set_bit(28, status);
    }

    pub fn get_irq_disable(&self) -> bool {
        self.value.bit(7)
    }

    pub fn set_irq_disable(&mut self, status: bool) {
        self.value.set_bit(7, status);
    }

    pub fn get_fiq_disable(&self) -> bool {
        self.value.bit(6)
    }

    pub fn set_fiq_disable(&mut self, status: bool) {
        self.value.set_bit(6, status);
    }

    pub fn get_state(&self) -> CpuState {
        self.value.bit(5).into()
    }

    pub fn set_state(&mut self, state: CpuState) {
        self.value.set_bit(5, state.into());
    }

    pub fn get_mode(&self) -> CpuMode {
        self.value.bit_range(0..5).into()
    }

    pub fn set_mode(&mut self, mode: CpuMode) {
        self.value.set_bit_range(0..5, mode.into());
    }
}
