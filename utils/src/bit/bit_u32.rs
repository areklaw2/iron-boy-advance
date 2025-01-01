use std::ops::Range;

use super::BitIndex;

impl BitIndex for u32 {
    fn bit(&self, index: usize) -> bool {
        if index >= 32 {
            panic!("Index out of bounds for u32");
        }
        (*self & (1 << index)) != 0
    }

    fn bit_range(&self, range: Range<usize>) -> Self {
        if range.start >= 32 || range.end > 32 {
            panic!("Range out of bounds for u32");
        }

        let mut mask = 0u32;
        for i in range.start..range.end {
            mask |= 1 << i;
        }
        (*self & mask) >> range.start
    }

    fn set_bit(&mut self, index: usize, value: bool) {
        if index >= 32 {
            panic!("Index out of bounds for u32");
        }

        *self &= !(1 << index);
        *self |= (value as u32) << index
    }

    fn set_bit_range(&mut self, range: Range<usize>, value: Self) {
        if range.start >= 32 || range.end > 32 {
            panic!("Range out of bounds for u32");
        }

        let mut mask = 0u32;
        for i in range.start..range.end {
            mask |= 1 << i;
        }
        *self = (*self & !mask) | (value << range.start);
    }
}

#[cfg(test)]
mod tests {
    use super::BitIndex;

    #[test]
    fn u32_bit() {
        let x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        assert_eq!(x.bit(5), true);
        assert_eq!(x.bit(0), false);
    }

    #[test]
    #[should_panic]
    fn u32_bit_panics() {
        let x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        x.bit(32);
    }

    #[test]
    fn u32_bit_range() {
        let x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        assert_eq!(x.bit_range(2..6), 0b1111);
    }

    #[test]
    #[should_panic]
    fn u32_bit_range_panics() {
        let x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        x.bit_range(0..33);
    }

    #[test]
    fn u32_set_bit() {
        let mut x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        assert_eq!(x.bit(6), false);
        x.set_bit(6, true);
        assert_eq!(x.bit(6), true);
    }

    #[test]
    #[should_panic]
    fn u32_set_bit_panics() {
        let mut x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        x.set_bit(32, true);
    }

    #[test]
    fn u32_set_bit_range() {
        let mut x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        assert_eq!(x.bit_range(4..8), 0b0011);
        x.set_bit_range(4..8, 0b1111);
        assert_eq!(x.bit_range(4..8), 0b1111);
    }

    #[test]
    #[should_panic]
    fn u32_set_bit_range_panics() {
        let mut x: u32 = 0b0011_1100_0011_1100_0011_1100_0011_1100;
        x.set_bit_range(5..33, 0b1111);
    }
}
