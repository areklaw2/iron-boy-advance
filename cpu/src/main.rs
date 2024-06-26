use bit::BitIndex;

fn main() {
    let mut x: u8 = 0b1000_1000;
    x.set_bit_range(0..7, 0000_0000);
    println!("{:#6b}", x)
}
