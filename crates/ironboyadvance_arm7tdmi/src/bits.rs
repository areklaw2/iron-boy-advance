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

#[cfg(test)]
mod tests {
    use super::SignExtend;

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
}
