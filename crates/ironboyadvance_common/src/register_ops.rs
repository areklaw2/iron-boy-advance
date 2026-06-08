use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

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

    /// Bits visible on a read. Bits cleared here always read back as 0
    /// (write-only fields). Defaults to every bit readable.
    fn read_mask(&self) -> T {
        !T::from(0)
    }

    /// Bits the CPU is allowed to change on a write. Bits cleared here keep
    /// their current value (read-only or hardware-owned fields). Defaults to
    /// every bit writable.
    fn write_mask(&self) -> T {
        !T::from(0)
    }

    fn read_byte(&self, address: u32) -> u8 {
        let bits = self.register() & self.read_mask();
        let byte_mask = (std::mem::size_of::<T>() - 1) as u32; // 1 for u16, 3 for u32
        let shift = ((address & byte_mask) * 8) as usize;
        ((bits >> shift) & T::from(0xFF)).try_into().unwrap_or(0)
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        let current = self.register();
        let byte_mask = (std::mem::size_of::<T>() - 1) as u32; // 1 for u16, 3 for u32
        let shift = ((address & byte_mask) * 8) as usize;
        let writable = self.write_mask() & (T::from(0xFF) << shift);
        let incoming = T::from(value) << shift;
        let bits = (current & !writable) | (incoming & writable);
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

    // Read mask clears bits 11/15 (write-only), write mask clears bits 0-3 (read-only).
    struct MaskedReg(u16);

    impl RegisterOps<u16> for MaskedReg {
        fn register(&self) -> u16 {
            self.0
        }

        fn write_register(&mut self, bits: u16) {
            self.0 = bits;
        }

        fn read_mask(&self) -> u16 {
            0x77FF
        }

        fn write_mask(&self) -> u16 {
            0xFFF0
        }
    }

    #[test]
    fn test_read_mask_clears_write_only_bits() {
        let value = MaskedReg(0xFFFF);
        // Low byte is fully readable; high byte drops bits 11 and 15.
        assert_eq!(value.read_byte(0x00000000), 0xFF);
        assert_eq!(value.read_byte(0x00000001), 0x77);
    }

    #[test]
    fn test_write_mask_protects_read_only_bits() {
        let mut value = MaskedReg(0x000F); // read-only bits 0-3 set
        value.write_byte(0x00000000, 0x00); // try to clear the whole low byte
        assert_eq!(value.register(), 0x000F); // read-only bits preserved
    }

    #[test]
    fn test_write_mask_allows_writable_bits() {
        let mut value = MaskedReg(0x0000);
        value.write_byte(0x00000000, 0xFF); // bits 4-7 writable, 0-3 protected
        assert_eq!(value.register(), 0x00F0);
        value.write_byte(0x00000001, 0xFF); // whole high byte writable
        assert_eq!(value.register(), 0xFFF0);
    }

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
