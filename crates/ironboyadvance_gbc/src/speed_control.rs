use bitfields::bitfield;
use ironboyadvance_common::memory::SystemMemoryAccess;
use ironboyadvance_sm83::GbSpeed;

const SPEED_SWITCH_UNUSED_BITS: u8 = 0x7E;

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct SpeedSwitch {
    switch_armed: bool,
    #[bits(6)]
    _not_used_1_6: u8,
    double_speed: bool,
}

pub struct SpeedController {
    speed_switch: SpeedSwitch,
}

impl SpeedController {
    pub fn new() -> Self {
        SpeedController {
            speed_switch: SpeedSwitch::from_bits(0),
        }
    }

    pub fn speed(&self) -> GbSpeed {
        match self.speed_switch.double_speed() {
            true => GbSpeed::Double,
            false => GbSpeed::Normal,
        }
    }

    pub fn change_speed(&mut self) -> bool {
        if !self.speed_switch.switch_armed() {
            return false;
        }

        self.speed_switch.set_double_speed(!self.speed_switch.double_speed());
        self.speed_switch.set_switch_armed(false);
        true
    }
}

impl SystemMemoryAccess for SpeedController {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF4D => self.speed_switch.into_bits() | SPEED_SWITCH_UNUSED_BITS,
            _ => panic!("Invalid byte read for SpeedController: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF4D => self.speed_switch.set_switch_armed(value & 0x01 != 0),
            _ => panic!("Invalid byte write for SpeedController: {:#06X}", address),
        }
    }
}
