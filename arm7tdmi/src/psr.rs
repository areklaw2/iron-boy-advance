use crate::disassembler::{Mode, State};
use getset::{CopyGetters, Setters};

#[derive(Debug, Copy, Clone, CopyGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct ProgramStatusRegister {
    negative: bool,
    zero: bool,
    carry: bool,
    overflow: bool,
    irq_disable: bool,
    fiq_disable: bool,
    state: State,
    mode: Mode,
}

impl ProgramStatusRegister {
    pub fn new(value: u32) -> Self {
        ProgramStatusRegister {
            negative: value & (1 << 31) != 0,
            zero: value & (1 << 30) != 0,
            carry: value & (1 << 29) != 0,
            overflow: value & (1 << 28) != 0,
            irq_disable: value & (1 << 7) != 0,
            fiq_disable: value & (1 << 6) != 0,
            state: (value & (1 << 5) != 0).into(),
            mode: (value & 0x1F).into(),
        }
    }

    pub fn value(&self) -> u32 {
        (self.negative as u32) << 31
            | (self.zero as u32) << 30
            | (self.carry as u32) << 29
            | (self.overflow as u32) << 28
            | (self.irq_disable as u32) << 7
            | (self.fiq_disable as u32) << 6
            | (self.state as u32) << 5
            | self.mode as u32
    }

    pub fn set_value(&mut self, value: u32) {
        self.negative = value & (1 << 31) != 0;
        self.zero = value & (1 << 30) != 0;
        self.carry = value & (1 << 29) != 0;
        self.overflow = value & (1 << 28) != 0;
        self.irq_disable = value & (1 << 7) != 0;
        self.fiq_disable = value & (1 << 6) != 0;
        self.state = (value & (1 << 5) != 0).into();
        self.mode = (value & 0x1F).into();
    }

    pub fn set_flags(&mut self, value: u32) {
        self.negative = value & (1 << 31) != 0;
        self.zero = value & (1 << 30) != 0;
        self.carry = value & (1 << 29) != 0;
        self.overflow = value & (1 << 28) != 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::disassembler::{Mode, State};

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
        assert_eq!(psr.state(), State::Thumb)
    }

    #[test]
    fn set_psr_state() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_state(State::Arm);
        assert_eq!(psr.state(), State::Arm)
    }

    #[test]
    fn get_psr_mode() {
        let psr = ProgramStatusRegister::new(0xFFFFFFFF);
        assert_eq!(psr.mode(), Mode::System)
    }

    #[test]
    fn set_psr_mode() {
        let mut psr = ProgramStatusRegister::new(0xFFFFFFFF);
        psr.set_mode(Mode::User);
        assert_eq!(psr.mode(), Mode::User);

        psr.set_mode(Mode::Fiq);
        assert_eq!(psr.mode(), Mode::Fiq);

        psr.set_mode(Mode::Irq);
        assert_eq!(psr.mode(), Mode::Irq);

        psr.set_mode(Mode::Supervisor);
        assert_eq!(psr.mode(), Mode::Supervisor);

        psr.set_mode(Mode::Abort);
        assert_eq!(psr.mode(), Mode::Abort);

        psr.set_mode(Mode::Undefined);
        assert_eq!(psr.mode(), Mode::Undefined);
    }

    #[test]
    #[should_panic]
    fn get_psr_mode_panics() {
        let psr = ProgramStatusRegister::new(0xFFFFFF15);
        psr.mode();
    }
}
