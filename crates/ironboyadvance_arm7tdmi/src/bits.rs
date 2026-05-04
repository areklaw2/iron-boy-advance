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
