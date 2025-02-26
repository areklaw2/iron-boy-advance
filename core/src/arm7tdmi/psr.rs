use bitfields::bitfield;

use super::disassembler::{CpuMode, CpuState};

#[bitfield(u32)]
#[derive(Copy, Clone)]
pub struct ProgramStatusRegister {
    #[bits(5)]
    cpu_mode: CpuMode,
    #[bits(1)]
    cpu_state: CpuState,
    fiq_disable: bool,
    irq_disable: bool,
    #[bits(20)]
    _reserved: u32,
    overflow: bool,
    carry: bool,
    zero: bool,
    negative: bool,
}

impl ProgramStatusRegister {
    pub fn flags(&self) -> u8 {
        (self.negative() as u8) << 3 | (self.zero() as u8) << 2 | (self.carry() as u8) << 1 | (self.overflow() as u8)
    }

    pub fn set_flags(&mut self, value: u8) {
        //Take first 4 bits
        let value = (value & !0x0F) >> 4;
        self.set_negative((value >> 3) != 0);
        self.set_zero((value >> 2) != 0);
        self.set_carry((value >> 1) != 0);
        self.set_overflow(value & 0x01 != 0);
    }
}

#[cfg(test)]
mod tests {

    use crate::arm7tdmi::disassembler::{CpuMode, CpuState};

    use super::ProgramStatusRegister;

    #[test]
    fn from_bits_psr() {
        let psr = ProgramStatusRegister::from_bits(0xFFFF_FFFF);
        assert_eq!(psr.into_bits(), 0xF000_00FF)
    }

    #[test]
    fn set_psr_flags() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFF_FF11);
        psr.set_flags(0xEF);
        assert_eq!(psr.into_bits(), 0xE000_0011);
        assert_eq!(psr.flags(), 0xE);

        psr.set_flags(0x00);
        assert_eq!(psr.into_bits(), 0x0000_0011);
        assert_eq!(psr.flags(), 0x0);
    }

    #[test]
    fn get_psr_negative() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.negative(), true)
    }

    #[test]
    fn set_psr_negative() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        psr.set_negative(false);
        assert_eq!(psr.negative(), false)
    }

    #[test]
    fn get_psr_zero() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.zero(), true)
    }

    #[test]
    fn set_psr_zero() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        psr.set_zero(false);
        assert_eq!(psr.zero(), false)
    }

    #[test]
    fn get_psr_carry() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.carry(), true)
    }

    #[test]
    fn set_psr_carry() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        psr.set_carry(false);
        assert_eq!(psr.carry(), false)
    }

    #[test]
    fn get_psr_overflow() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.overflow(), true)
    }

    #[test]
    fn set_psr_overflow() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        psr.set_overflow(false);
        assert_eq!(psr.overflow(), false)
    }

    #[test]
    fn get_psr_irq_disable() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.irq_disable(), true)
    }

    #[test]
    fn set_psr_irq_disable() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        psr.set_irq_disable(false);
        assert_eq!(psr.irq_disable(), false)
    }

    #[test]
    fn get_psr_fiq_disable() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.fiq_disable(), true)
    }

    #[test]
    fn set_psr_fiq_disable() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        psr.set_fiq_disable(false);
        assert_eq!(psr.fiq_disable(), false)
    }

    #[test]
    fn get_psr_state() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.cpu_state(), CpuState::Thumb)
    }

    #[test]
    fn set_psr_state() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        psr.set_cpu_state(CpuState::Arm);
        assert_eq!(psr.cpu_state(), CpuState::Arm)
    }

    #[test]
    fn get_psr_mode() {
        let psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
        assert_eq!(psr.cpu_mode(), CpuMode::System)
    }

    #[test]
    fn set_psr_mode() {
        let mut psr = ProgramStatusRegister::from_bits(0xFFFFFFFF);
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
        let psr = ProgramStatusRegister::from_bits(0xFFFFFF15);
        psr.cpu_mode();
    }
}
