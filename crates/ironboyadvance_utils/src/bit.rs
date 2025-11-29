use std::mem::size_of;
use std::ops::RangeInclusive;

pub trait BitOps {
    fn bit(&self, index: usize) -> bool;
    fn set_bit(&mut self, index: usize, value: bool);
    fn bits(&self, range: RangeInclusive<usize>) -> Self;
    fn byte(&self, index: usize) -> u8;
    fn set_byte(&mut self, index: usize, value: u8);
}

macro_rules! impl_bitops {
    ($($t:ty),+ $(,)?) => {
        $(
            impl BitOps for $t {
                fn bit(&self, index: usize) -> bool {
                    debug_assert!(index < size_of::<$t>() * 8);
                    let mask = 1 << index;
                    (self & mask) != 0
                }

                fn set_bit(&mut self, index: usize, value: bool) {
                    debug_assert!(index < size_of::<$t>() * 8);
                    let mask = 1 << index;
                    if value {
                        *self |= mask;
                    } else {
                        *self &= !mask;
                    }
                }

                fn bits(&self, range: RangeInclusive<usize>) -> Self {
                    let start = *range.start();
                    let end = *range.end();
                    debug_assert!(end < size_of::<$t>() * 8);
                    debug_assert!(start <= end);

                    let length = end - start + 1;
                    let bit_width = size_of::<$t>() * 8;
                    let mask = if length >= bit_width {
                        <$t>::MAX
                    } else {
                        ((1 as $t) << length) - 1
                    };
                    (self >> start) & mask
                }

                fn byte(&self, index: usize) -> u8 {
                    debug_assert!(index < size_of::<$t>());
                    ((self >> (index * 8)) & 0xFF) as u8
                }

                fn set_byte(&mut self, index: usize, value: u8) {
                    debug_assert!(index < size_of::<$t>());
                    let shift = index * 8;
                    let mask = !(0xFF << shift);
                    *self = (*self & mask) | ((value as $t) << shift);
                }
            }
        )+
    };
}

impl_bitops!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_bit_operations() {
        let mut value: u8 = 0b10110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(2), true);
        assert_eq!(value.bit(3), false);
        assert_eq!(value.bit(4), true);
        assert_eq!(value.bit(7), false);

        value.set_bit(0, true);
        assert_eq!(value, 0b10111);
        value.set_bit(4, false);
        assert_eq!(value, 0b00111);
        value.set_bit(7, true);
        assert_eq!(value, 0b10000111);

        assert_eq!(value.bits(0..=2), 0b111);
        assert_eq!(value.bits(7..=7), 0b1);
        assert_eq!(value.bits(0..=7), value);

        assert_eq!(value.byte(0), value);
        let mut val: u8 = 0;
        val.set_byte(0, 0xFF);
        assert_eq!(val, 0xFF);
    }

    #[test]
    fn u16_bit_operations() {
        let mut value: u16 = 0b1010110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(15), false);

        value.set_bit(0, true);
        assert_eq!(value, 0b1010111);
        value.set_bit(15, true);
        assert_eq!(value, 0b1000000001010111);

        assert_eq!(value.bits(0..=3), 0b0111);
        assert_eq!(value.bits(15..=15), 0b1);

        let value: u16 = 0x1234;
        assert_eq!(value.byte(0), 0x34);
        assert_eq!(value.byte(1), 0x12);

        let mut value: u16 = 0;
        value.set_byte(0, 0xAB);
        assert_eq!(value, 0x00AB);
        value.set_byte(1, 0xCD);
        assert_eq!(value, 0xCDAB);
    }

    #[test]
    fn u32_bit_operations() {
        let mut value: u32 = 0b10110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(31), false);

        value.set_bit(31, true);
        assert_eq!(value.bit(31), true);

        assert_eq!(value.bits(1..=3), 0b011);

        let value: u32 = 0x12345678;
        assert_eq!(value.byte(0), 0x78);
        assert_eq!(value.byte(1), 0x56);
        assert_eq!(value.byte(2), 0x34);
        assert_eq!(value.byte(3), 0x12);

        let mut value: u32 = 0;
        value.set_byte(0, 0xAA);
        value.set_byte(1, 0xBB);
        value.set_byte(2, 0xCC);
        value.set_byte(3, 0xDD);
        assert_eq!(value, 0xDDCCBBAA);
    }

    #[test]
    fn u64_bit_operations() {
        let mut value: u64 = 0b10110;

        assert_eq!(value.bit(0), false);
        assert_eq!(value.bit(1), true);
        assert_eq!(value.bit(63), false);

        value.set_bit(63, true);
        assert_eq!(value.bit(63), true);

        assert_eq!(value.bits(1..=4), 0b1011);

        let value: u64 = 0x0123456789ABCDEF;
        assert_eq!(value.byte(0), 0xEF);
        assert_eq!(value.byte(7), 0x01);

        let mut val: u64 = 0;
        for i in 0..8 {
            val.set_byte(i, i as u8);
        }
        assert_eq!(val, 0x0706050403020100);
    }

    #[test]
    fn set_bit_clear() {
        let mut value: u32 = 0b1111;

        value.set_bit(0, false);
        assert_eq!(value, 0b1110);

        value.set_bit(1, false);
        assert_eq!(value, 0b1100);

        value.set_bit(2, false);
        value.set_bit(3, false);
        assert_eq!(value, 0);
    }

    #[test]
    fn set_and_get_bit() {
        let mut value: u16 = 0;

        for i in 0..16 {
            value.set_bit(i, true);
            assert_eq!(value.bit(i), true);
        }

        for i in 0..16 {
            value.set_bit(i, false);
            assert_eq!(value.bit(i), false);
        }
    }

    #[test]
    fn set_and_get_byte() {
        let mut value: u32 = 0;

        for i in 0..4 {
            let test_byte = (i * 17) as u8;
            value.set_byte(i, test_byte);
            assert_eq!(value.byte(i), test_byte, "byte {} mismatch", i);
        }
    }

    #[test]
    fn bits_single_bit() {
        let value: u8 = 0b10110;
        assert_eq!(value.bits(0..=0), if value.bit(0) { 1 } else { 0 });
        assert_eq!(value.bits(1..=1), if value.bit(1) { 1 } else { 0 });
        assert_eq!(value.bits(4..=4), if value.bit(4) { 1 } else { 0 });
    }

    #[test]
    fn bits_edge_cases() {
        let value: u8 = 0b11001010;
        assert_eq!(value.bits(0..=0), 0);
        assert_eq!(value.bits(7..=7), 1);
        assert_eq!(value.bits(0..=7), value);
        assert_eq!(value.bits(2..=5), 0b0010);
    }
}
