use bitfields::bitfield;
use getset::{CopyGetters, MutGetters, Setters};

use crate::GbMode;

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Flags {
    #[bits(4)]
    _reserved: u8,
    carry: bool,
    half_carry: bool,
    subtraction: bool,
    zero: bool,
}

#[derive(Debug, CopyGetters, MutGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct Registers {
    a: u8,
    #[getset(get_mut = "pub")]
    f: Flags,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

impl Registers {
    pub fn new(skip_boot: bool, mode: GbMode) -> Self {
        if !skip_boot {
            return Registers {
                a: 0,
                f: Flags::from_bits_with_defaults(0),
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0,
                l: 0,
                pc: 0,
                sp: 0,
            };
        }

        match mode {
            GbMode::Monochrome => Registers {
                a: 0x01,
                f: Flags::from_bits_with_defaults(0b1011_0000),
                b: 0x00,
                c: 0x13,
                d: 0x00,
                e: 0xD8,
                h: 0x01,
                l: 0x4D,
                pc: 0x0100,
                sp: 0xFFFE,
            },
            GbMode::Color => Registers {
                a: 0x11,
                f: Flags::from_bits_with_defaults(0b1000_0000),
                b: 0x00,
                c: 0x00,
                d: 0xFF,
                e: 0x56,
                h: 0x00,
                l: 0x0D,
                pc: 0x0100,
                sp: 0xFFFE,
            },
            GbMode::ColorAsMonochrome => Registers {
                a: 0x11,
                f: Flags::from_bits_with_defaults(0b1000_0000),
                b: 0x00,
                c: 0x00,
                d: 0x00,
                e: 0x08,
                h: 0x00,
                l: 0x7C,
                pc: 0x0100,
                sp: 0xFFFE,
            },
        }
    }

    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f.into_bits() as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = Flags::from((value & 0x00F0) as u8)
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8
    }

    pub fn decrement_hl(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl(hl - 1);
        hl
    }

    pub fn increment_hl(&mut self) -> u16 {
        let hl = self.hl();
        self.set_hl(hl + 1);
        hl
    }
}

#[cfg(test)]
mod tests {

    use super::Flags;

    #[test]
    fn from_bits_flags() {
        let flags = Flags::from_bits_with_defaults(0xFF);
        assert_eq!(flags.into_bits(), 0xFF)
    }

    #[test]
    fn get_flags_carry() {
        let flags = Flags::from_bits_with_defaults(0xFF);
        assert!(flags.carry())
    }

    #[test]
    fn set_flags_carry() {
        let mut flags = Flags::from_bits_with_defaults(0xFF);
        flags.set_carry(false);
        assert!(!flags.carry())
    }

    #[test]
    fn get_flags_half_carry() {
        let flags = Flags::from_bits_with_defaults(0xFF);
        assert!(flags.half_carry())
    }

    #[test]
    fn set_flags_half_carry() {
        let mut flags = Flags::from_bits_with_defaults(0xFF);
        flags.set_half_carry(false);
        assert!(!flags.half_carry())
    }

    #[test]
    fn get_flags_subtraction() {
        let flags = Flags::from_bits_with_defaults(0xFF);
        assert!(flags.subtraction())
    }

    #[test]
    fn set_flags_subtraction() {
        let mut flags = Flags::from_bits_with_defaults(0xFF);
        flags.set_subtraction(false);
        assert!(!flags.subtraction())
    }

    #[test]
    fn get_flags_zero() {
        let flags = Flags::from_bits_with_defaults(0xFF);
        assert!(flags.zero())
    }

    #[test]
    fn set_flags_zero() {
        let mut flags = Flags::from_bits_with_defaults(0xFF);
        flags.set_zero(false);
        assert!(!flags.zero())
    }
}
