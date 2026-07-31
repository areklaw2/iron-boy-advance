use std::path::PathBuf;

use ironboyadvance_common::emulator::Emulator;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GbcError {}

pub struct GameBoyColor {}

impl GameBoyColor {
    pub fn new(rom_path: PathBuf, rom_buffer: Vec<u8>, show_logs: bool) -> Result<GameBoyColor, GbcError> {
        let gba = GameBoyColor {};
        Ok(gba)
    }

    pub fn cycle(&mut self) {}

    fn handle_events(&mut self) -> bool {
        true
    }
}

impl Emulator for GameBoyColor {
    fn run(&mut self, cycles: usize, overshoot: usize) -> usize {
        todo!()
    }

    fn frame_buffer(&self) -> &[u32] {
        todo!()
    }

    fn audio_buffer(&self) -> &[(f32, f32)] {
        todo!()
    }

    fn clear_audio_buffer(&mut self) {
        todo!()
    }

    fn handle_pressed_buttons(&mut self, input: u16) {
        todo!()
    }
}
