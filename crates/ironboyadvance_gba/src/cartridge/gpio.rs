use getset::CopyGetters;

use crate::cartridge::rtc::Rtc;

#[derive(CopyGetters)]
pub struct Gpio {
    data: u8,
    direction: u8,
    #[getset(get_copy = "pub(crate)")]
    readable: bool,
    rtc: Option<Rtc>,
}

impl Gpio {
    pub fn new(rtc: Option<Rtc>) -> Gpio {
        Gpio {
            data: 0,
            direction: 0,
            readable: false,
            rtc,
        }
    }

    pub fn in_range(address: u32) -> bool {
        (0x080000C4..=0x080000C8 + 1).contains(&address)
    }

    pub fn read_16(&self, address: u32) -> u16 {
        match address & !1 {
            0x080000C4 => self.read_pins() as u16,
            0x080000C6 => self.direction as u16,
            0x080000C8 => self.readable as u16,
            _ => 0,
        }
    }

    pub fn write_16(&mut self, address: u32, value: u16) {
        match address & !1 {
            0x080000C4 => {
                self.data = value as u8 & 0xF;
                self.write_pins();
            }
            0x080000C6 => {
                self.direction = value as u8 & 0xF;
                self.write_pins();
            }
            0x080000C8 => self.readable = value & 1 != 0,
            _ => {}
        }
    }

    fn read_pins(&self) -> u8 {
        let device_pins = match &self.rtc {
            Some(rtc) => rtc.read_pins(),
            None => 0,
        };

        (self.data & self.direction | device_pins & !self.direction) & 0xF
    }

    fn write_pins(&mut self) {
        let driven_pins = self.data & self.direction;
        if let Some(rtc) = &mut self.rtc {
            rtc.write_pins(driven_pins);
        }
    }
}
