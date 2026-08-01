// Order matters: discriminant is used as a bit position by consumers that pack button
// presses into the GBA KEYINPUT register format (A=bit0, B=bit1, ... R=bit8, L=bit9).
// The low byte is also the Game Boy P1 layout: bits 0-3 are the action row, bits 4-7
// the direction row.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum KeypadButton {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
    R,
    L,
}
