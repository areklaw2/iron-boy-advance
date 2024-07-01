use core::bus::{Bus, MemoryAccess};

use dissassembler::{CpuMode, CpuState};
use psr::ProgramStatusRegister;

mod arm;
mod dissassembler;
mod psr;
mod thumb;

const SP: usize = 13;
const LR: usize = 14;

pub struct Cpu {
    general_registers: Vec<Vec<u32>>,
    pc: u32,
    cpsr: ProgramStatusRegister,
    spsrs: Vec<ProgramStatusRegister>,
    fetched_instruction: u32,
    decoded_instruction: u32,
    bus: Bus,
}

impl MemoryAccess for Cpu {
    fn read_byte(&self, address: u32) -> u8 {
        self.bus.read_byte(address)
    }

    fn write_byte(&mut self, address: u32, value: u8) {
        self.bus.write_byte(address, value)
    }
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Cpu {
            general_registers: build_general_registers(),
            pc: 0,
            cpsr: ProgramStatusRegister::new(0),
            spsrs: vec![ProgramStatusRegister::new(0); 5],
            fetched_instruction: 0,
            decoded_instruction: 0,
            bus,
        }
    }

    fn increment_program_counter(&mut self) {
        match self.cpsr.state() {
            CpuState::ARM => self.pc.wrapping_add(4),
            CpuState::Thumb => self.pc.wrapping_add(2),
        };
    }

    pub fn cycle(&mut self) {
        let state = self.cpsr.state();
        match state {
            CpuState::ARM => {
                let pc = self.pc & !0b11;
                let executed_instruction = self.decoded_instruction;
                self.decoded_instruction = self.fetched_instruction;
                self.fetched_instruction = self.bus.read_word(pc);

                //decode and execute instruction
            }
            CpuState::Thumb => {
                let pc = self.pc & !0b01;
            }
        }
    }
}

fn build_general_registers() -> Vec<Vec<u32>> {
    let mut general_registers = Vec::new();
    for i in 0..16 {
        match i {
            0..=7 => general_registers.push(vec![0; 1]),
            8..=12 => general_registers.push(vec![0; 2]),
            13 | 14 => general_registers.push(vec![0; 6]),
            _ => general_registers.push(vec![0; 1]),
        }
    }
    general_registers
}
