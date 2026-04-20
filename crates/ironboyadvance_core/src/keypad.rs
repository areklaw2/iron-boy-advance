use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::io_registers::RegisterOps;

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
struct KeyInput {
    a: bool,
    b: bool,
    select: bool,
    start: bool,
    right: bool,
    left: bool,
    up: bool,
    down: bool,
    r: bool,
    l: bool,
    #[bits(6)]
    not_used_10_15: u8,
}

impl RegisterOps<u16> for KeyInput {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
struct KeyControl {
    #[bits(10)]
    buttons: u16,
    #[bits(4)]
    not_used_10_13: u8,
    irq_enable_flag: bool,
    irq_condition: bool,
}

impl RegisterOps<u16> for KeyControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum KeypadButton {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
    R,
    L,
}

pub struct Keypad {
    key_input: KeyInput,
    key_control: KeyControl,
}

impl Keypad {
    pub fn new() -> Self {
        Self {
            key_input: KeyInput::from_bits(0x03FF),
            key_control: KeyControl::from_bits(0x0000),
        }
    }

    pub fn set_key_input(&mut self, input: u16) {
        self.key_input = KeyInput::from_bits(input);
    }

    pub fn keypad_interrupt_raised(&self) -> bool {
        if self.key_control.irq_enable_flag() {
            let pressed = !self.key_input.into_bits() & 0x03FF;
            let selected = self.key_control.into_bits() & 0x03FF;
            return match self.key_control.irq_condition() {
                // AND: all selected buttons pressed
                true => (pressed & selected) == selected,
                // OR: any selected buttons pressed
                false => (pressed & selected) != 0,
            };
        }
        false
    }
}

impl SystemMemoryAccess for Keypad {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // KEYINPUT
            0x04000130..=0x04000131 => self.key_input.read_byte(address),
            // KEYCNT
            0x04000132..=0x04000133 => self.key_control.read_byte(address),
            _ => panic!("Invalid byte read for Keypad: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // KEYINPUT
            0x04000130..=0x04000131 => {}
            // KEYCNT
            0x04000132..=0x04000133 => self.key_control.write_byte(address, value),
            _ => panic!("Invalid byte write for Keypad: {:#010X}", address),
        }
    }
}
