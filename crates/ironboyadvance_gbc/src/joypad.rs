use ironboyadvance_common::memory::SystemMemoryAccess;

pub struct Joypad {
    button_input: u16,
    select: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            button_input: 0x03FF,
            select: 0x30,
        }
    }

    pub fn set_button_input(&mut self, button_input: u16) -> bool {
        let previous = self.selected_input();
        self.button_input = button_input;

        previous & !self.selected_input() != 0
    }

    pub fn selected_pressed(&self) -> bool {
        self.selected_input() != 0x0F
    }

    fn selected_input(&self) -> u8 {
        let mut input = 0x0F;

        if self.select & 0x20 == 0 {
            input &= self.button_input as u8 & 0x0F;
        }

        if self.select & 0x10 == 0 {
            input &= (self.button_input >> 4) as u8 & 0x0F;
        }

        input
    }
}

impl SystemMemoryAccess for Joypad {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF00 => 0xC0 | self.select | self.selected_input(),
            _ => panic!("Invalid byte read for Joypad: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF00 => self.select = value & 0x30,
            _ => panic!("Invalid byte write for Joypad: {:#06X}", address),
        }
    }
}
