use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

use getset::{Getters, MutGetters, Setters};
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use tracing::debug;

use crate::{interrupt_control::InterruptController, ppu::Ppu, system_control::SystemController};

#[derive(Getters, MutGetters, Setters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct IoRegisters {
    ppu: Ppu,
    interrupt_controller: InterruptController,
    system_controller: SystemController,
}

impl IoRegisters {
    pub fn new() -> Self {
        IoRegisters {
            ppu: Ppu::new(),
            interrupt_controller: InterruptController::new(),
            system_controller: SystemController::new(),
        }
    }
}

impl SystemMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // PPU
            0x04000000..=0x04000057 => self.ppu.read_8(address),
            // Interrupt Control
            0x04000200..=0x04000203 | 0x04000208..=0x0400020B => self.interrupt_controller.read_8(address),
            // System Control
            0x04000204..=0x04000207 | 0x04000300..=0x04000301 | 0x04000410 => self.system_controller.read_8(address),
            0x04000000..=0x04FFFFFF => self.system_controller.read_8(address), // Mirroring for 0x04000800
            // Access Memory
            0x05000000..=0x05FFFFFF => self.ppu.read_8(address),
            0x06000000..=0x06FFFFFF => self.ppu.read_8(address),
            0x07000000..=0x07FFFFFF => self.ppu.read_8(address),
            _ => {
                debug!("Read byte not implemented for I/O register: {:#010X}", address);
                0
            }
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // PPU
            0x04000000..=0x04000005 | 0x04000008..=0x04000057 => self.ppu.write_8(address, value),
            // Interrupt Control
            0x04000200..=0x04000203 | 0x04000208..=0x0400020B => self.interrupt_controller.write_8(address, value),
            // System Control
            0x04000204..=0x04000207 | 0x04000300..=0x04000301 | 0x04000410 => self.system_controller.write_8(address, value),
            0x04000000..=0x04FFFFFF => self.system_controller.write_8(address, value), // Mirroring for 0x04000800
            // Access Memory
            0x05000000..=0x05FFFFFF => self.ppu.write_8(address, value),
            0x06000000..=0x06FFFFFF => self.ppu.write_8(address, value),
            0x07000000..=0x07FFFFFF => self.ppu.write_8(address, value),
            _ => debug!(
                "Write byte not implemented for I/O register: {:#010X}, value: {:#04X}",
                address, value
            ),
        }
    }
}

pub trait RegisterOps<T>
where
    T: Copy
        + From<u8>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + Shl<usize, Output = T>
        + Shr<usize, Output = T>
        + Not<Output = T>
        + TryInto<u8>,
{
    fn register(&self) -> T;
    fn write_register(&mut self, bits: T);

    fn read_byte(&self, address: u32) -> u8 {
        let bits = self.register();
        let byte_mask = (std::mem::size_of::<T>() - 1) as u32; // 1 for u16, 3 for u32
        let shift = ((address & byte_mask) * 8) as usize;
        ((bits >> shift) & T::from(0xFF)).try_into().unwrap_or(0)
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        let mut bits = self.register();
        let byte_mask = (std::mem::size_of::<T>() - 1) as u32; // 1 for u16, 3 for u32
        let shift = ((address & byte_mask) * 8) as usize;
        let mask = !(T::from(0xFF) << shift);
        bits = (bits & mask) | (T::from(value) << shift);
        self.write_register(bits);
    }
}

impl RegisterOps<u16> for u16 {
    fn register(&self) -> u16 {
        *self
    }

    fn write_register(&mut self, bits: u16) {
        *self = bits;
    }
}

impl RegisterOps<u32> for u32 {
    fn register(&self) -> u32 {
        *self
    }

    fn write_register(&mut self, bits: u32) {
        *self = bits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_reg_16_byte_low() {
        let value: u16 = 0x1234;
        assert_eq!(value.read_byte(0x04000200), 0x34);
        assert_eq!(value.read_byte(0x00000000), 0x34);
    }

    #[test]
    fn test_read_reg_16_byte_high() {
        let value: u16 = 0x1234;
        assert_eq!(value.read_byte(0x04000201), 0x12);
        assert_eq!(value.read_byte(0x00000001), 0x12);
    }

    #[test]
    fn test_write_reg_16_byte_low() {
        let mut current: u16 = 0x1234;
        current.write_byte(0x04000200, 0xAB);
        assert_eq!(current, 0x12AB);
    }

    #[test]
    fn test_write_reg_16_byte_high() {
        let mut current: u16 = 0x1234;
        current.write_byte(0x04000201, 0xAB);
        assert_eq!(current, 0xAB34);
    }

    #[test]
    fn test_write_reg_16_byte_preserves_other_byte() {
        let mut current: u16 = 0xFFFF;
        current.write_byte(0x00000000, 0x00);
        assert_eq!(current, 0xFF00);

        current = 0xFFFF;
        current.write_byte(0x00000001, 0x00);
        assert_eq!(current, 0x00FF);
    }

    #[test]
    fn test_read_reg_32_byte_byte0() {
        let value: u32 = 0x12345678;
        assert_eq!(value.read_byte(0x04000800), 0x78);
        assert_eq!(value.read_byte(0x00000000), 0x78);
    }

    #[test]
    fn test_read_reg_32_byte_byte1() {
        let value: u32 = 0x12345678;
        assert_eq!(value.read_byte(0x04000801), 0x56);
        assert_eq!(value.read_byte(0x00000001), 0x56);
    }

    #[test]
    fn test_read_reg_32_byte_byte2() {
        let value: u32 = 0x12345678;
        assert_eq!(value.read_byte(0x04000802), 0x34);
        assert_eq!(value.read_byte(0x00000002), 0x34);
    }

    #[test]
    fn test_read_reg_32_byte_byte3() {
        let value: u32 = 0x12345678;
        assert_eq!(value.read_byte(0x04000803), 0x12);
        assert_eq!(value.read_byte(0x00000003), 0x12);
    }

    #[test]
    fn test_write_u32_byte_byte0() {
        let mut current: u32 = 0x12345678;
        current.write_byte(0x04000800, 0xAA);
        assert_eq!(current, 0x123456AA);
    }

    #[test]
    fn test_write_u32_byte_byte1() {
        let mut current: u32 = 0x12345678;
        current.write_byte(0x04000801, 0xBB);
        assert_eq!(current, 0x1234BB78);
    }

    #[test]
    fn test_write_u32_byte_byte2() {
        let mut current: u32 = 0x12345678;
        current.write_byte(0x04000802, 0xCC);
        assert_eq!(current, 0x12CC5678);
    }

    #[test]
    fn test_write_u32_byte_byte3() {
        let mut current: u32 = 0x12345678;
        current.write_byte(0x04000803, 0xDD);
        assert_eq!(current, 0xDD345678);
    }

    #[test]
    fn test_write_u32_byte_preserves_other_bytes() {
        let mut current: u32 = 0xFFFFFFFF;
        current.write_byte(0x00000000, 0x00);
        assert_eq!(current, 0xFFFFFF00);

        current = 0xFFFFFFFF;
        current.write_byte(0x00000001, 0x00);
        assert_eq!(current, 0xFFFF00FF);

        current = 0xFFFFFFFF;
        current.write_byte(0x00000002, 0x00);
        assert_eq!(current, 0xFF00FFFF);

        current = 0xFFFFFFFF;
        current.write_byte(0x00000003, 0x00);
        assert_eq!(current, 0x00FFFFFF);
    }

    #[test]
    fn test_write_read_16_byte() {
        let original: u16 = 0xABCD;
        let mut value: u16 = 0x0000;

        value.write_byte(0x00000000, 0xCD);
        assert_eq!(value.read_byte(0x00000000), 0xCD);

        value.write_byte(0x00000001, 0xAB);
        assert_eq!(value.read_byte(0x00000001), 0xAB);
        assert_eq!(value, original);
    }

    #[test]
    fn test_write_read_32_byte() {
        let original: u32 = 0x12345678;
        let mut value: u32 = 0x00000000;

        // Write all bytes
        value.write_byte(0x00000000, 0x78);
        value.write_byte(0x00000001, 0x56);
        value.write_byte(0x00000002, 0x34);
        value.write_byte(0x00000003, 0x12);

        // Read them back
        assert_eq!(value.read_byte(0x00000000), 0x78);
        assert_eq!(value.read_byte(0x00000001), 0x56);
        assert_eq!(value.read_byte(0x00000002), 0x34);
        assert_eq!(value.read_byte(0x00000003), 0x12);
        assert_eq!(value, original);
    }
}
