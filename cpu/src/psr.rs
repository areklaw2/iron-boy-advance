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

#[cfg(test)]
mod tests {
    use crate::mode::{CpuMode, CpuState};

    use super::ProgramStatusRegister;

    #[test]
    fn get_psr() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get(), 0xF00000FF)
    }

    #[test]
    fn set_psr() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set(0xEFFFFFEE);
        assert_eq!(psr.get(), 0xE00000EE);
    }

    #[test]
    fn set_psr_flags() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFF11);
        psr.set_flags(0xEFFF4FEE);
        assert_eq!(psr.get(), 0xE0000011);

        psr.set_flags(0x01FF); // has leading zeroes, equivalent to setting flags to zero
        assert_eq!(psr.get(), 0x00000011);
    }

    #[test]
    fn get_psr_n_flag() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_n_flag(), true)
    }

    #[test]
    fn set_psr_n_flag() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_n_flag(false);
        assert_eq!(psr.get_n_flag(), false)
    }

    #[test]
    fn get_psr_z_flag() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_z_flag(), true)
    }

    #[test]
    fn set_psr_z_flag() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_z_flag(false);
        assert_eq!(psr.get_z_flag(), false)
    }

    #[test]
    fn get_psr_c_flag() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_c_flag(), true)
    }

    #[test]
    fn set_psr_c_flag() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_c_flag(false);
        assert_eq!(psr.get_c_flag(), false)
    }

    #[test]
    fn get_psr_v_flag() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_v_flag(), true)
    }

    #[test]
    fn set_psr_v_flag() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_v_flag(false);
        assert_eq!(psr.get_v_flag(), false)
    }

    #[test]
    fn get_psr_irq_disable() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_irq_disable(), true)
    }

    #[test]
    fn set_psr_irq_disable() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_irq_disable(false);
        assert_eq!(psr.get_irq_disable(), false)
    }

    #[test]
    fn get_psr_fiq_disable() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_fiq_disable(), true)
    }

    #[test]
    fn set_psr_fiq_disable() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_fiq_disable(false);
        assert_eq!(psr.get_fiq_disable(), false)
    }

    #[test]
    fn get_psr_state() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_state(), CpuState::Thumb)
    }

    #[test]
    fn set_psr_state() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_state(CpuState::ARM);
        assert_eq!(psr.get_state(), CpuState::ARM)
    }

    #[test]
    fn get_psr_mode() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.get_mode(), CpuMode::System)
    }

    #[test]
    fn set_psr_mode() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_mode(CpuMode::User);
        assert_eq!(psr.get_mode(), CpuMode::User);

        psr.set_mode(CpuMode::FIQ);
        assert_eq!(psr.get_mode(), CpuMode::FIQ);

        psr.set_mode(CpuMode::IRQ);
        assert_eq!(psr.get_mode(), CpuMode::IRQ);

        psr.set_mode(CpuMode::Supervisor);
        assert_eq!(psr.get_mode(), CpuMode::Supervisor);

        psr.set_mode(CpuMode::Abort);
        assert_eq!(psr.get_mode(), CpuMode::Abort);

        psr.set_mode(CpuMode::Undefined);
        assert_eq!(psr.get_mode(), CpuMode::Undefined);
    }

    #[test]
    #[should_panic]
    fn get_psr_mode_panics() {
        let psr = ProgramStatusRegister::new(0xFFFFFF15);
        psr.get_mode();
    }
}
