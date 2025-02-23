use bitvec::{field::BitField, order::Lsb0, view::BitView};
use utils::get_set;

use super::disassembler::{CpuMode, CpuState};

#[derive(Debug, Copy, Clone)]
// remove get set write proc macro
pub struct ProgramStatusRegister {
    negative: bool,
    zero: bool,
    carry: bool,
    overflow: bool,
    irq_disable: bool,
    fiq_disable: bool,
    cpu_state: CpuState,
    cpu_mode: CpuMode,
}

impl ProgramStatusRegister {
    pub fn new(value: u32) -> Self {
        let value = value & !0x0FFF_FF00;
        let bits = value.view_bits::<Lsb0>();

        ProgramStatusRegister {
            negative: bits[31],
            zero: bits[30],
            carry: bits[29],
            overflow: bits[28],
            irq_disable: bits[7],
            fiq_disable: bits[6],
            cpu_state: bits[5].into(),
            cpu_mode: bits[0..=4].load::<u32>().into(),
        }
    }

    get_set!(negative, set_negative, bool);
    get_set!(zero, set_zero, bool);
    get_set!(carry, set_carry, bool);
    get_set!(overflow, set_overflow, bool);
    get_set!(irq_disable, set_irq_disable, bool);
    get_set!(fiq_disable, set_fiq_disable, bool);
    get_set!(cpu_state, set_cpu_state, CpuState);
    get_set!(cpu_mode, set_cpu_mode, CpuMode);

    pub fn flags(&self) -> u32 {
        let value = (self.negative as u32) << 31
            | (self.zero as u32) << 30
            | (self.carry as u32) << 29
            | (self.overflow as u32) << 28;
        value >> 28
    }

    pub fn set_flags(&mut self, value: u32) {
        let value = value & !0x0FFF_FF00;
        let bits = value.view_bits::<Lsb0>();
        self.negative = bits[31];
        self.zero = bits[30];
        self.carry = bits[29];
        self.overflow = bits[28];
    }

    pub fn value(&self) -> u32 {
        (self.negative as u32) << 31
            | (self.zero as u32) << 30
            | (self.carry as u32) << 29
            | (self.overflow as u32) << 28
            | (self.irq_disable as u32) << 7
            | (self.fiq_disable as u32) << 6
            | (self.cpu_state as u32) << 5
            | (self.cpu_mode as u32)
    }

    pub fn set_value(&mut self, value: u32) {
        let value = value & !0x0FFFFF00;
        let bits = value.view_bits::<Lsb0>();
        self.negative = bits[31];
        self.zero = bits[30];
        self.carry = bits[29];
        self.overflow = bits[28];
        self.irq_disable = bits[7];
        self.fiq_disable = bits[6];
        self.cpu_state = bits[5].into();
        self.cpu_mode = bits[0..=4].load::<u32>().into();
    }
}

#[cfg(test)]
mod tests {

    use crate::arm7tdmi::disassembler::{CpuMode, CpuState};

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
        assert_eq!(psr.cpu_state(), CpuState::Thumb)
    }

    #[test]
    fn set_psr_state() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_cpu_state(CpuState::Arm);
        assert_eq!(psr.cpu_state(), CpuState::Arm)
    }

    #[test]
    fn get_psr_mode() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.cpu_mode(), CpuMode::System)
    }

    #[test]
    fn set_psr_mode() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_cpu_mode(CpuMode::User);
        assert_eq!(psr.cpu_mode(), CpuMode::User);

        psr.set_cpu_mode(CpuMode::Fiq);
        assert_eq!(psr.cpu_mode(), CpuMode::Fiq);

        psr.set_cpu_mode(CpuMode::Irq);
        assert_eq!(psr.cpu_mode(), CpuMode::Irq);

        psr.set_cpu_mode(CpuMode::Supervisor);
        assert_eq!(psr.cpu_mode(), CpuMode::Supervisor);

        psr.set_cpu_mode(CpuMode::Abort);
        assert_eq!(psr.cpu_mode(), CpuMode::Abort);

        psr.set_cpu_mode(CpuMode::Undefined);
        assert_eq!(psr.cpu_mode(), CpuMode::Undefined);
    }

    #[test]
    #[should_panic]
    fn get_psr_mode_panics() {
        let psr = ProgramStatusRegister::new(0xFFFFFF15);
        psr.cpu_mode();
    }
}
