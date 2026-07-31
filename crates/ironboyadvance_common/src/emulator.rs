pub trait Emulator {
    fn run(&mut self, cycles: usize, overshoot: usize) -> usize;
    fn frame_buffer(&self) -> &[u32];
    fn audio_buffer(&self) -> &[(f32, f32)];
    fn clear_audio_buffer(&mut self);
    fn handle_pressed_buttons(&mut self, input: u16);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System {
    Gba,
    Gbc,
    Gb,
}

pub fn detect_system(rom: &[u8]) -> Option<System> {
    if is_gba_rom(rom) {
        return Some(System::Gba);
    }

    if is_gb_rom(rom) {
        return Some(match rom[0x143] {
            0x80 | 0xC0 => System::Gbc,
            _ => System::Gb,
        });
    }

    None
}

fn is_gba_rom(rom: &[u8]) -> bool {
    rom.len() > 0xBD && rom[0xBD] == gba_complement_checksum(rom)
}

fn gba_complement_checksum(rom: &[u8]) -> u8 {
    let mut checksum = 0u8;
    for byte in &rom[0xA0..=0xBC] {
        checksum = checksum.wrapping_sub(*byte);
    }
    checksum.wrapping_sub(0x19)
}

fn is_gb_rom(rom: &[u8]) -> bool {
    rom.len() > 0x14D && rom[0x14D] == gb_header_checksum(rom)
}

fn gb_header_checksum(rom: &[u8]) -> u8 {
    let mut checksum = 0u8;
    for byte in &rom[0x134..=0x14C] {
        checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
    }
    checksum
}
