use ironboyadvance_arm7tdmi_jit::WATCHED_MEMORY_CHUNK_SIZE;
use ironboyadvance_common::memory::SystemMemoryAccess;
use tracing::debug;

const WRAM_BOARD_SIZE: usize = 0x40000;
const WRAM_CHIP_SIZE: usize = 0x8000;

pub struct Memory {
    wram_board: Vec<u8>,
    wram_chip: Vec<u8>,
    written_board_chunks: Vec<bool>,
    written_chip_chunks: Vec<bool>,
    dirty: bool,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            wram_board: vec![0; WRAM_BOARD_SIZE],
            wram_chip: vec![0; WRAM_CHIP_SIZE],
            written_board_chunks: vec![false; WRAM_BOARD_SIZE / WATCHED_MEMORY_CHUNK_SIZE],
            written_chip_chunks: vec![false; WRAM_CHIP_SIZE / WATCHED_MEMORY_CHUNK_SIZE],
            dirty: false,
        }
    }

    pub fn watch_memory_range(&mut self, address: u32, length: u32) {
        match address {
            0x02000000..=0x02FFFFFF => {
                let offset = (address & 0x3FFFF) as usize;
                let start = offset / WATCHED_MEMORY_CHUNK_SIZE;
                let end = (offset + length as usize - 1) / WATCHED_MEMORY_CHUNK_SIZE;
                self.written_board_chunks[start..=end].fill(true);
            }
            0x03000000..=0x03FFFFFF => {
                let offset = (address & 0x7FFF) as usize;
                let start = offset / WATCHED_MEMORY_CHUNK_SIZE;
                let end = (offset + length as usize - 1) / WATCHED_MEMORY_CHUNK_SIZE;
                self.written_chip_chunks[start..=end].fill(true);
            }
            _ => debug!("Cannot watch in this range {:08X}", address),
        }
    }

    pub fn take_watched_write(&mut self) -> bool {
        if !self.dirty {
            return false;
        }

        self.dirty = false;
        self.written_board_chunks.fill(false);
        self.written_chip_chunks.fill(false);
        true
    }
}

impl SystemMemoryAccess for Memory {
    type Address = u32;

    fn read_8(&self, address: u32) -> u8 {
        match address {
            0x02000000..=0x02FFFFFF => self.wram_board[(address & 0x3FFFF) as usize],
            0x03000000..=0x03FFFFFF => self.wram_chip[(address & 0x7FFF) as usize],
            _ => panic!("Invalid byte read for Memory: {:08X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            0x02000000..=0x02FFFFFF => {
                let offset = (address & 0x3FFFF) as usize;
                let current_value = self.wram_board[offset];
                let chunk = offset / WATCHED_MEMORY_CHUNK_SIZE;
                if self.written_board_chunks[chunk] && current_value != value {
                    self.dirty = true;
                }
                self.wram_board[offset] = value;
            }
            0x03000000..=0x03FFFFFF => {
                let offset = (address & 0x7FFF) as usize;
                let current_value = self.wram_chip[offset];
                let chunk = offset / WATCHED_MEMORY_CHUNK_SIZE;
                if self.written_chip_chunks[chunk] && current_value != value {
                    self.dirty = true;
                }
                self.wram_chip[offset] = value;
            }
            _ => panic!("Invalid byte write for Memory: {:08X}", address),
        }
    }
}
