use utils::bit::BitIndex;

use crate::disassembler::{CpuMode, CpuState};

#[derive(Debug, Copy, Clone)]
// remove get set write proc macro
pub struct ProgramStatusRegister {
    value: u32,
}

impl ProgramStatusRegister {
    pub fn new(value: u32) -> Self {
        ProgramStatusRegister {
            value: value & !0x0FFFFF00,
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn set_value(&mut self, value: u32) {
        self.value = value & !0x0FFFFF00;
    }

    pub fn set_flags(&mut self, value: u32) {
        self.value &= !0xF0000000;
        self.value |= 0xF0000000 & value;
    }

    pub fn negative(&self) -> bool {
        self.value.bit(31)
    }

    pub fn set_negative(&mut self, status: bool) {
        self.value.set_bit(31, status);
    }

    pub fn zero(&self) -> bool {
        self.value.bit(30)
    }

    pub fn set_zero(&mut self, status: bool) {
        self.value.set_bit(30, status);
    }

    pub fn carry(&self) -> bool {
        self.value.bit(29)
    }

    pub fn set_carry(&mut self, status: bool) {
        self.value.set_bit(29, status);
    }

    pub fn overflow(&self) -> bool {
        self.value.bit(28)
    }

    pub fn set_overflow(&mut self, status: bool) {
        self.value.set_bit(28, status);
    }

    pub fn irq_disable(&self) -> bool {
        self.value.bit(7)
    }

    pub fn set_irq_disable(&mut self, status: bool) {
        self.value.set_bit(7, status);
    }

    pub fn fiq_disable(&self) -> bool {
        self.value.bit(6)
    }

    pub fn set_fiq_disable(&mut self, status: bool) {
        self.value.set_bit(6, status);
    }

    pub fn state(&self) -> CpuState {
        self.value.bit(5).into()
    }

    pub fn set_state(&mut self, state: CpuState) {
        self.value.set_bit(5, state.into());
    }

    pub fn mode(&self) -> CpuMode {
        self.value.bit_range(0..5).into()
    }

    pub fn set_mode(&mut self, mode: CpuMode) {
        self.value.set_bit_range(0..5, mode.into());
    }
}

#[cfg(test)]
mod tests {
    use crate::disassembler::{CpuMode, CpuState};

    use super::ProgramStatusRegister;

    #[test]
    fn get_psr() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.value(), 0xF00000FF)
    }

    #[test]
    fn set_psr() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_value(0xEFFFFF3B);
        assert_eq!(psr.value(), 0xE000003B);
    }

    #[test]
    fn set_psr_flags() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFF11);
        psr.set_flags(0xEFFF4FEE);
        assert_eq!(psr.value(), 0xE0000011);

        psr.set_flags(0x01FF); // has leading zeroes, equivalent to setting flags to zero
        assert_eq!(psr.value(), 0x00000011);
    }

    #[test]
    fn get_psr_negative() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.negative(), true)
    }

    #[test]
    fn set_psr_negative() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_negative(false);
        assert_eq!(psr.negative(), false)
    }

    #[test]
    fn get_psr_zero() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.zero(), true)
    }

    #[test]
    fn set_psr_zero() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_zero(false);
        assert_eq!(psr.zero(), false)
    }

    #[test]
    fn get_psr_carry() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.carry(), true)
    }

    #[test]
    fn set_psr_carry() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_carry(false);
        assert_eq!(psr.carry(), false)
    }

    #[test]
    fn get_psr_overflow() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.overflow(), true)
    }

    #[test]
    fn set_psr_overflow() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_overflow(false);
        assert_eq!(psr.overflow(), false)
    }

    #[test]
    fn get_psr_irq_disable() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.irq_disable(), true)
    }

    #[test]
    fn set_psr_irq_disable() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_irq_disable(false);
        assert_eq!(psr.irq_disable(), false)
    }

    #[test]
    fn get_psr_fiq_disable() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.fiq_disable(), true)
    }

    #[test]
    fn set_psr_fiq_disable() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_fiq_disable(false);
        assert_eq!(psr.fiq_disable(), false)
    }

    #[test]
    fn get_psr_state() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.state(), CpuState::Thumb)
    }

    #[test]
    fn set_psr_state() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_state(CpuState::Arm);
        assert_eq!(psr.state(), CpuState::Arm)
    }

    #[test]
    fn get_psr_mode() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.mode(), CpuMode::System)
    }

    #[test]
    fn set_psr_mode() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_mode(CpuMode::User);
        assert_eq!(psr.mode(), CpuMode::User);

        psr.set_mode(CpuMode::Fiq);
        assert_eq!(psr.mode(), CpuMode::Fiq);

        psr.set_mode(CpuMode::Irq);
        assert_eq!(psr.mode(), CpuMode::Irq);

        psr.set_mode(CpuMode::Supervisor);
        assert_eq!(psr.mode(), CpuMode::Supervisor);

        psr.set_mode(CpuMode::Abort);
        assert_eq!(psr.mode(), CpuMode::Abort);

        psr.set_mode(CpuMode::Undefined);
        assert_eq!(psr.mode(), CpuMode::Undefined);
    }

    #[test]
    #[should_panic]
    fn get_psr_mode_panics() {
        let psr = ProgramStatusRegister::new(0xFFFFFF15);
        psr.mode();
    }
}
