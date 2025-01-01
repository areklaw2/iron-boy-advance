use std::ops::Range;

pub mod bit_u16;
pub mod bit_u32;
pub mod bit_u8;

pub trait BitIndex {
    fn bit(&self, index: usize) -> bool;
    fn bit_range(&self, range: Range<usize>) -> Self;
    fn set_bit(&mut self, index: usize, value: bool);
    fn set_bit_range(&mut self, range: Range<usize>, value: Self);
}
