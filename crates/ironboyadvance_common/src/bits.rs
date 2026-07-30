use std::mem::size_of;
use std::ops::RangeInclusive;

/// Sign-extend the bottom `bits` bits of an unsigned value, interpreting them
/// as two's-complement, into a full `i32`.
///
/// Example: `0x1FFu16.sign_extend(9) == -1` because bit 8 is the 9-bit sign bit.
pub trait SignExtend {
    fn sign_extend(self, bits: u32) -> i32;
}

impl SignExtend for u32 {
    fn sign_extend(self, bits: u32) -> i32 {
        let shift = 32 - bits;
        ((self << shift) as i32) >> shift
    }
}

impl SignExtend for u16 {
    fn sign_extend(self, bits: u32) -> i32 {
        (self as u32).sign_extend(bits)
    }
}

pub trait BitOps {
    fn bit(&self, index: usize) -> bool;
    fn set_bit(&mut self, index: usize, value: bool);
    fn bits(&self, range: RangeInclusive<usize>) -> Self;
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
            }
        )+
    };
}

impl_bitops!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::{BitOps, SignExtend};

    #[test]
    fn docstring_example() {
        assert_eq!(0x1FFu16.sign_extend(9), -1);
    }

    #[test]
    fn zero_is_zero_at_any_width() {
        assert_eq!(0u32.sign_extend(8), 0);
        assert_eq!(0u32.sign_extend(16), 0);
        assert_eq!(0u32.sign_extend(28), 0);
        assert_eq!(0u32.sign_extend(32), 0);
    }

    #[test]
    fn positive_value_unchanged_when_sign_bit_clear() {
        assert_eq!(0x01u32.sign_extend(8), 1);
        assert_eq!(0x7Fu32.sign_extend(8), 127);
        assert_eq!(0x0000_0001u32.sign_extend(28), 1);
        assert_eq!(0x07FF_FFFFu32.sign_extend(28), 134_217_727);
    }

    #[test]
    fn all_ones_within_field_is_negative_one() {
        assert_eq!(0xFFu32.sign_extend(8), -1);
        assert_eq!(0x1FFu32.sign_extend(9), -1);
        assert_eq!(0xFFFFu32.sign_extend(16), -1);
        assert_eq!(0x0FFF_FFFFu32.sign_extend(28), -1);
    }

    #[test]
    fn min_negative_at_each_width() {
        assert_eq!(0x80u32.sign_extend(8), -128);
        assert_eq!(0x100u32.sign_extend(9), -256);
        assert_eq!(0x8000u32.sign_extend(16), -32_768);
        assert_eq!(0x0800_0000u32.sign_extend(28), -134_217_728);
    }

    #[test]
    fn max_positive_at_each_width() {
        assert_eq!(0x7Fu32.sign_extend(8), 127);
        assert_eq!(0xFFu32.sign_extend(9), 255);
        assert_eq!(0x7FFFu32.sign_extend(16), 32_767);
        assert_eq!(0x07FF_FFFFu32.sign_extend(28), 134_217_727);
    }

    #[test]
    fn bits_above_field_are_discarded() {
        assert_eq!(0xF800_0000u32.sign_extend(28), -134_217_728);
        assert_eq!(0xFFFF_FFFFu32.sign_extend(28), -1);
        assert_eq!(0xF000_0001u32.sign_extend(28), 1);
    }

    #[test]
    fn full_width_acts_like_plain_cast() {
        assert_eq!(0xFFFF_FFFFu32.sign_extend(32), -1);
        assert_eq!(0x8000_0000u32.sign_extend(32), i32::MIN);
        assert_eq!(0x7FFF_FFFFu32.sign_extend(32), i32::MAX);
    }

    #[test]
    fn u16_impl_matches_u32_impl() {
        assert_eq!(0x80u16.sign_extend(8), (0x80u32).sign_extend(8));
        assert_eq!(0x1FFu16.sign_extend(9), (0x1FFu32).sign_extend(9));
        assert_eq!(0xFFFFu16.sign_extend(16), (0xFFFFu32).sign_extend(16));
    }

    #[test]
    fn sixteen_bit_intermediate_values() {
        assert_eq!(0x0100u16.sign_extend(16), 256);
        assert_eq!(0xFF00u16.sign_extend(16), -256);
        assert_eq!(0x0080u16.sign_extend(16), 128);
    }

    #[test]
    fn twenty_eight_bit_intermediate_values() {
        assert_eq!(0x0FFF_FF00u32.sign_extend(28), -256);
        assert_eq!(0x0000_0080u32.sign_extend(28), 128);
    }

    #[test]
    fn u8_bit_operations() {
        let mut value: u8 = 0b10110;

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(value.bit(2));
        assert!(!value.bit(3));
        assert!(value.bit(4));
        assert!(!value.bit(7));

        value.set_bit(0, true);
        assert_eq!(value, 0b10111);
        value.set_bit(4, false);
        assert_eq!(value, 0b00111);
        value.set_bit(7, true);
        assert_eq!(value, 0b10000111);

        assert_eq!(value.bits(0..=2), 0b111);
        assert_eq!(value.bits(7..=7), 0b1);
        assert_eq!(value.bits(0..=7), value);
    }

    #[test]
    fn u16_bit_operations() {
        let mut value: u16 = 0b1010110;

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(!value.bit(15));

        value.set_bit(0, true);
        assert_eq!(value, 0b1010111);
        value.set_bit(15, true);
        assert_eq!(value, 0b1000000001010111);

        assert_eq!(value.bits(0..=3), 0b0111);
        assert_eq!(value.bits(15..=15), 0b1);
    }

    #[test]
    fn u32_bit_operations() {
        let mut value: u32 = 0b10110;

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(!value.bit(31));

        value.set_bit(31, true);
        assert!(value.bit(31));

        assert_eq!(value.bits(1..=3), 0b011);
    }

    #[test]
    fn u64_bit_operations() {
        let mut value: u64 = 0b10110;

        assert!(!value.bit(0));
        assert!(value.bit(1));
        assert!(!value.bit(63));

        value.set_bit(63, true);
        assert!(value.bit(63));

        assert_eq!(value.bits(1..=4), 0b1011);
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
            assert!(value.bit(i));
        }

        for i in 0..16 {
            value.set_bit(i, false);
            assert!(!value.bit(i));
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
